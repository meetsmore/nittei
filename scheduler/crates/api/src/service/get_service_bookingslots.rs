use crate::error::NettuError;
use crate::shared::usecase::{execute, UseCase};
use actix_web::{web, HttpRequest, HttpResponse};
use futures::future::join_all;
use nettu_scheduler_api_structs::get_service_bookingslots::*;
use nettu_scheduler_domain::{
    booking_slots::{
        get_service_bookingslots, validate_bookingslots_query, validate_slots_interval,
        BookingQueryError, BookingSlotsOptions, BookingSlotsQuery, ServiceBookingSlots,
        UserFreeEvents,
    },
    get_free_busy, BusyCalendar, Calendar, CompatibleInstances, EventInstance,
    ServiceResource, ServiceWithUsers, TimePlan, TimeSpan, ID,
};
use nettu_scheduler_infra::{
    google_calendar::GoogleCalendarProvider, FreeBusyProviderQuery, NettuContext,
};
use tracing::{info, warn};

fn handle_error(e: UseCaseErrors) -> NettuError {
    match e {
        UseCaseErrors::InvalidDate(msg) => {
            NettuError::BadClientData(format!(
                "Invalid datetime: {}. Should be YYYY-MM-DD, e.g. January 1. 2020 => 2020-1-1",
                msg
            ))
        }
        UseCaseErrors::InvalidTimezone(msg) => {
            NettuError::BadClientData(format!(
                "Invalid timezone: {}. It should be a valid IANA TimeZone.",
                msg
            ))
        }
        UseCaseErrors::InvalidInterval => {
            NettuError::BadClientData(
                "Invalid interval specified. It should be between 10 - 60 minutes inclusively and be specified as milliseconds.".into()
            )
        }
        UseCaseErrors::InvalidTimespan => {
            NettuError::BadClientData("The provided start and end is invalid".into())
        }
        UseCaseErrors::ServiceNotFound => NettuError::NotFound("Service was not found".into())
    }
}

pub async fn get_service_bookingslots_controller(
    _http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let query_params = query_params.0;
    let _service_id = path_params.service_id.clone();
    let usecase = GetServiceBookingSlotsUseCase {
        service_id: path_params.0.service_id,
        iana_tz: query_params.iana_tz,
        start_date: query_params.start_date,
        end_date: query_params.end_date,
        duration: query_params.duration,
        interval: query_params.interval,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.booking_slots)))
        .map_err(handle_error)
}

#[derive(Debug)]
pub(crate) struct GetServiceBookingSlotsUseCase {
    pub service_id: ID,
    pub start_date: String,
    pub end_date: String,
    pub iana_tz: Option<String>,
    pub duration: i64,
    pub interval: i64,
}

impl Into<NettuError> for UseCaseErrors {
    fn into(self) -> NettuError {
        handle_error(self)
    }
}

#[derive(Debug)]
pub(crate) struct UseCaseRes {
    pub booking_slots: ServiceBookingSlots,
    pub service: ServiceWithUsers,
}

#[derive(Debug)]
pub(crate) enum UseCaseErrors {
    ServiceNotFound,
    InvalidInterval,
    InvalidTimespan,
    InvalidDate(String),
    InvalidTimezone(String),
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetServiceBookingSlotsUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    const NAME: &'static str = "GetServiceBookingSlots";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Errors> {
        if !validate_slots_interval(self.interval) {
            return Err(UseCaseErrors::InvalidInterval);
        }

        let query = BookingSlotsQuery {
            start_date: self.start_date.clone(),
            end_date: self.end_date.clone(),
            iana_tz: self.iana_tz.clone(),
            interval: self.interval,
            duration: self.duration,
        };
        let booking_timespan = match validate_bookingslots_query(&query) {
            Ok(t) => t,
            Err(e) => match e {
                BookingQueryError::InvalidInterval => return Err(UseCaseErrors::InvalidInterval),
                BookingQueryError::InvalidTimespan => return Err(UseCaseErrors::InvalidTimespan),
                BookingQueryError::InvalidDate(d) => return Err(UseCaseErrors::InvalidDate(d)),
                BookingQueryError::InvalidTimezone(d) => {
                    return Err(UseCaseErrors::InvalidTimezone(d))
                }
            },
        };
        let timezone: chrono_tz::Tz = query
            .iana_tz
            .unwrap_or_else(|| "UTC".into())
            .parse()
            .unwrap();

        let service = match ctx.repos.services.find_with_users(&self.service_id).await {
            Some(s) => s,
            None => return Err(UseCaseErrors::ServiceNotFound),
        };

        let mut usecase_futures: Vec<_> = Vec::with_capacity(service.users.len());

        let timespan = TimeSpan::new(booking_timespan.start_ts, booking_timespan.end_ts);
        if timespan.greater_than(ctx.config.booking_slots_query_duration_limit) {
            return Err(UseCaseErrors::InvalidTimespan);
        }

        for user in &service.users {
            let timespan = timespan.clone();
            usecase_futures.push(self.get_bookable_times(user, timespan, ctx));
        }

        let users_free_events = join_all(usecase_futures).await;

        let booking_slots = get_service_bookingslots(
            users_free_events,
            &BookingSlotsOptions {
                interval: self.interval,
                duration: self.duration,
                end_ts: booking_timespan.end_ts,
                start_ts: booking_timespan.start_ts,
            },
        );

        Ok(UseCaseRes {
            booking_slots: ServiceBookingSlots::new(booking_slots, timezone),
            service,
        })
    }
}

impl GetServiceBookingSlotsUseCase {
    async fn get_user_availability(
        &self,
        user: &ServiceResource,
        user_calendars: &[Calendar],
        timespan: &TimeSpan,
        ctx: &NettuContext,
    ) -> CompatibleInstances {
        let empty = CompatibleInstances::new(Vec::new());
        match &user.availability {
            TimePlan::Calendar(id) => {
                let calendar = match user_calendars.iter().find(|cal| cal.id == *id) {
                    Some(cal) => cal,
                    None => {
                        return empty;
                    }
                };
                let all_calendar_events = ctx
                    .repos
                    .events
                    .find_by_calendar(&id, Some(&timespan))
                    .await
                    .unwrap_or_default();

                let all_event_instances = all_calendar_events
                    .iter()
                    .map(|e| e.expand(Some(&timespan), &calendar.settings))
                    .flatten()
                    .collect::<Vec<_>>();

                get_free_busy(all_event_instances).free
            }
            TimePlan::Schedule(id) => match ctx.repos.schedules.find(&id).await {
                Some(schedule) if schedule.user_id == user.user_id => schedule.freebusy(&timespan),
                _ => empty,
            },
            TimePlan::Empty => empty,
        }
    }

    async fn get_user_busy(
        &self,
        user: &ServiceResource,
        busy_calendars: &[&Calendar],
        timespan: &TimeSpan,
        ctx: &NettuContext,
    ) -> CompatibleInstances {
        let mut busy_events: Vec<EventInstance> = Vec::new();
        let all_service_resources = ctx.repos.service_users.find_by_user(&user.user_id).await;

        for cal in busy_calendars {
            match ctx
                .repos
                .events
                .find_by_calendar(&cal.id, Some(&timespan))
                .await
            {
                Ok(calendar_events) => {
                    let mut calendar_busy_events = calendar_events
                        .into_iter()
                        .filter(|e| e.busy)
                        .map(|e| {
                            let mut instances = e.expand(Some(&timespan), &cal.settings);

                            // Add buffer to instances if event is a service event
                            if let Some(service_id) = e.service_id {
                                let service_resource = all_service_resources
                                    .iter()
                                    .find(|s| s.service_id == service_id);
                                if let Some(service_resource) = service_resource {
                                    let buffer_after_in_millis =
                                        service_resource.buffer_after * 60 * 1000;
                                    let buffer_before_in_millis =
                                        service_resource.buffer_before * 60 * 1000;
                                    for instance in instances.iter_mut() {
                                        instance.end_ts += buffer_after_in_millis;
                                        instance.start_ts -= buffer_before_in_millis;
                                    }
                                }
                            }

                            instances
                        })
                        .flatten()
                        .collect::<Vec<_>>();

                    busy_events.append(&mut calendar_busy_events);
                }
                Err(e) => {
                    warn!("Unable to fetch user calendars: {}", e);
                }
            }
        }

        let google_busy_calendars = user
            .busy
            .iter()
            .filter(|busy_cal| match busy_cal {
                BusyCalendar::Google(_) => true,
                _ => false,
            })
            .collect::<Vec<_>>();

        if !google_busy_calendars.is_empty() {
            // TODO: no unwrap
            let mut user = ctx
                .repos
                .users
                .find(&user.user_id)
                .await
                .expect("User to be found");
            match GoogleCalendarProvider::new(&mut user, ctx).await {
                Ok(google_calendar_provider) => {
                    let query = FreeBusyProviderQuery {
                        calendar_ids: google_busy_calendars
                            .iter()
                            .map(|cal| match cal {
                                BusyCalendar::Google(id) => id.clone(),
                                _ => unreachable!("Nettu calendars should be filtered away"),
                            })
                            .collect(),
                        end: timespan.end(),
                        start: timespan.start(),
                    };
                    info!("Going to query freebusy: {:?}", query);
                    let google_busy = google_calendar_provider.freebusy(query).await;
                    for google_busy_event in google_busy.inner() {
                        busy_events.push(google_busy_event);
                    }
                }
                Err(_) => {}
            }
        }

        // This should be optimized later
        CompatibleInstances::new(busy_events)
    }

    /// Ensure that calendar timespan fits within user settings for when
    /// it should be bookable
    fn parse_calendar_timespan(
        user: &ServiceResource,
        mut timespan: TimeSpan,
        ctx: &NettuContext,
    ) -> Result<TimeSpan, ()> {
        let first_available =
            ctx.sys.get_timestamp_millis() + user.closest_booking_time * 60 * 1000;
        if timespan.start() < first_available {
            timespan = TimeSpan::new(first_available, timespan.end());
        }
        if let Some(furthest_booking_time) = user.furthest_booking_time {
            let last_available = furthest_booking_time * 60 * 1000 + ctx.sys.get_timestamp_millis();
            if last_available < timespan.end() {
                if last_available <= timespan.start() {
                    return Err(());
                }
                timespan = TimeSpan::new(timespan.start(), last_available);
            }
        }

        if timespan.greater_than(ctx.config.booking_slots_query_duration_limit) {
            Err(())
        } else {
            Ok(timespan)
        }
    }

    /// Finds the bookable times for a `User`.
    async fn get_bookable_times(
        &self,
        service_resource: &ServiceResource,
        mut timespan: TimeSpan,
        ctx: &NettuContext,
    ) -> UserFreeEvents {
        let empty = UserFreeEvents {
            free_events: CompatibleInstances::new(Vec::new()),
            user_id: service_resource.user_id.clone(),
        };

        match Self::parse_calendar_timespan(service_resource, timespan, ctx) {
            Ok(parsed_timespan) => timespan = parsed_timespan,
            Err(_) => return empty,
        }

        let user_calendars = ctx
            .repos
            .calendars
            .find_by_user(&service_resource.user_id)
            .await;
        let busy_calendars = user_calendars
            .iter()
            .filter(|cal| {
                service_resource
                    .busy
                    .iter()
                    .find(|busy_calendar| match busy_calendar {
                        BusyCalendar::Nettu(id) if *id == cal.id => true,
                        _ => false,
                    })
                    .is_some()
            })
            .collect::<Vec<_>>();

        let mut free_events = self
            .get_user_availability(service_resource, &user_calendars, &timespan, ctx)
            .await;

        let busy_events = self
            .get_user_busy(service_resource, &busy_calendars, &timespan, ctx)
            .await;

        free_events.remove_instances(&busy_events, 0);

        UserFreeEvents {
            free_events,
            user_id: service_resource.user_id.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;
    use chrono::prelude::*;
    use chrono::Utc;
    use nettu_scheduler_domain::{
        Account, Calendar, CalendarEvent, RRuleOptions, Service, ServiceResource, User,
    };
    use nettu_scheduler_infra::{setup_context, ISys};

    struct TestContext {
        ctx: NettuContext,
        service: Service,
        account: Account,
    }

    struct DummySys {}

    impl ISys for DummySys {
        fn get_timestamp_millis(&self) -> i64 {
            0
        }
    }

    async fn setup() -> TestContext {
        let mut ctx = setup_context().await;
        ctx.sys = Arc::new(DummySys {});

        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let service = Service::new(account.id.clone());
        ctx.repos.services.insert(&service).await.unwrap();

        TestContext {
            ctx,
            service,
            account,
        }
    }

    async fn setup_service_users(ctx: &NettuContext, service: &mut Service, account_id: &ID) {
        let user1 = User::new(account_id.clone());
        let user2 = User::new(account_id.clone());
        ctx.repos.users.insert(&user1).await.unwrap();
        ctx.repos.users.insert(&user2).await.unwrap();
        let mut resource1 = ServiceResource {
            user_id: user1.id.clone(),
            service_id: service.id.clone(),
            buffer_after: 0,
            buffer_before: 0,
            availability: TimePlan::Empty,
            busy: Vec::new(),
            closest_booking_time: 0,
            furthest_booking_time: None,
        };
        let mut resource2 = ServiceResource {
            user_id: user2.id.clone(),
            service_id: service.id.clone(),
            buffer_after: 0,
            buffer_before: 0,
            availability: TimePlan::Empty,
            busy: Vec::new(),
            closest_booking_time: 0,
            furthest_booking_time: None,
        };

        let calendar_user_1 = Calendar::new(&resource1.user_id, &account_id);
        resource1.availability = TimePlan::Calendar(calendar_user_1.id.clone());
        let calendar_user_2 = Calendar::new(&resource2.user_id, &account_id);
        resource2.availability = TimePlan::Calendar(calendar_user_2.id.clone());

        ctx.repos.calendars.insert(&calendar_user_1).await.unwrap();
        ctx.repos.calendars.insert(&calendar_user_2).await.unwrap();

        let availability_event1 = CalendarEvent {
            id: Default::default(),
            account_id: account_id.clone(),
            busy: false,
            calendar_id: calendar_user_1.id,
            duration: 1000 * 60 * 60,
            end_ts: 0,
            exdates: Vec::new(),
            recurrence: None,
            start_ts: 1000 * 60 * 60,
            user_id: resource1.user_id.to_owned(),
            reminder: None,
            service_id: None,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
            synced_events: Default::default(),
        };
        let availability_event2 = CalendarEvent {
            id: ID::default(),
            account_id: account_id.clone(),
            busy: false,
            calendar_id: calendar_user_2.id.clone(),
            duration: 1000 * 60 * 60,
            end_ts: 0,
            exdates: Vec::new(),
            recurrence: None,
            start_ts: 1000 * 60 * 60,
            user_id: resource2.user_id.to_owned(),
            reminder: None,
            service_id: None,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
            synced_events: Default::default(),
        };
        let mut availability_event3 = CalendarEvent {
            id: ID::default(),
            account_id: account_id.clone(),
            busy: false,
            calendar_id: calendar_user_2.id,
            duration: 1000 * 60 * 105,
            end_ts: 0,
            exdates: Vec::new(),
            recurrence: None,
            start_ts: 1000 * 60 * 60 * 4,
            user_id: resource1.user_id.to_owned(),
            reminder: None,
            service_id: None,
            metadata: Default::default(),
            updated: Default::default(),
            created: Default::default(),
            synced_events: Default::default(),
        };
        let recurrence = RRuleOptions {
            ..Default::default()
        };
        availability_event3.set_recurrence(recurrence, &calendar_user_2.settings, true);

        ctx.repos.events.insert(&availability_event1).await.unwrap();
        ctx.repos.events.insert(&availability_event2).await.unwrap();
        ctx.repos.events.insert(&availability_event3).await.unwrap();

        ctx.repos.service_users.insert(&resource1).await.unwrap();
        ctx.repos.service_users.insert(&resource2).await.unwrap();
    }

    #[actix_web::main]
    #[test]
    async fn get_service_bookingslots() {
        let TestContext {
            ctx,
            service,
            account: _,
        } = setup().await;

        let mut usecase = GetServiceBookingSlotsUseCase {
            start_date: "2010-1-1".into(),
            end_date: "2010-1-1".into(),
            duration: 1000 * 60 * 60,
            iana_tz: Utc.to_string().into(),
            interval: 1000 * 60 * 15,
            service_id: service.id,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        assert!(res.unwrap().booking_slots.dates.is_empty());
    }

    #[actix_web::main]
    #[test]
    async fn get_bookingslots_with_multiple_users_in_service() {
        let TestContext {
            ctx,
            mut service,
            account,
        } = setup().await;
        setup_service_users(&ctx, &mut service, &account.id).await;

        let mut usecase = GetServiceBookingSlotsUseCase {
            start_date: "2010-1-1".into(),
            end_date: "2010-1-1".into(),
            duration: 1000 * 60 * 60,
            iana_tz: Utc.to_string().into(),
            interval: 1000 * 60 * 15,
            service_id: service.id.clone(),
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let mut booking_slots = res.unwrap().booking_slots;
        assert_eq!(booking_slots.dates.len(), 1);
        let booking_slots = booking_slots.dates.remove(0).slots;

        assert_eq!(booking_slots.len(), 4);
        for i in 0..4 {
            assert_eq!(booking_slots[i].duration, usecase.duration);
            assert_eq!(booking_slots[i].user_ids.len(), 1);
            assert_eq!(
                booking_slots[i].start,
                Utc.ymd(2010, 1, 1)
                    .and_hms(4, 15 * i as u32, 0)
                    .timestamp_millis()
            );
        }

        let mut usecase = GetServiceBookingSlotsUseCase {
            start_date: "1970-1-1".into(),
            end_date: "1970-1-1".into(),
            duration: 1000 * 60 * 60,
            iana_tz: Utc.to_string().into(),
            interval: 1000 * 60 * 15,
            service_id: service.id,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let mut booking_slots = res.unwrap().booking_slots;
        assert_eq!(booking_slots.dates.len(), 1);
        let booking_slots = booking_slots.dates.remove(0).slots;

        assert_eq!(booking_slots.len(), 5);
        assert_eq!(booking_slots[0].user_ids.len(), 2);
        for i in 0..5 {
            assert_eq!(booking_slots[i].duration, usecase.duration);
            if i > 0 {
                assert_eq!(booking_slots[i].user_ids.len(), 1);
                assert_eq!(
                    booking_slots[i].start,
                    Utc.ymd(1970, 1, 1)
                        .and_hms(4, 15 * (i - 1) as u32, 0)
                        .timestamp_millis()
                );
            }
        }
    }
}
