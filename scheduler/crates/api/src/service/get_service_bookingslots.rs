use actix_web::{web, HttpRequest, HttpResponse};
use chrono::TimeDelta;
use futures::future::join_all;
use nittei_api_structs::get_service_bookingslots::*;
use nittei_domain::{
    booking_slots::{
        get_service_bookingslots,
        validate_bookingslots_query,
        validate_slots_interval,
        BookingQueryError,
        BookingSlotsOptions,
        BookingSlotsQuery,
        ServiceBookingSlots,
        UserFreeEvents,
    },
    get_free_busy,
    BusyCalendar,
    Calendar,
    CompatibleInstances,
    EventInstance,
    ServiceMultiPersonOptions,
    ServiceResource,
    ServiceWithUsers,
    TimePlan,
    TimeSpan,
    Tz,
    ID,
};
use nittei_infra::{
    google_calendar::GoogleCalendarProvider,
    outlook_calendar::OutlookCalendarProvider,
    FreeBusyProviderQuery,
    NitteiContext,
};
use tracing::{error, warn};

use crate::{
    error::NitteiError,
    shared::usecase::{execute, UseCase},
    user::parse_vec_query_value,
};

pub async fn get_service_bookingslots_controller(
    _http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    mut path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let query_params = query_params.0;
    let _service_id = path_params.service_id.clone();

    let host_user_ids = parse_vec_query_value(&query_params.host_user_ids);
    let usecase = GetServiceBookingSlotsUseCase {
        service_id: std::mem::take(&mut path_params.service_id),
        timezone: query_params.timezone,
        start_date: query_params.start_date,
        end_date: query_params.end_date,
        duration: query_params.duration,
        interval: query_params.interval,
        host_user_ids,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.booking_slots)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub(crate) struct GetServiceBookingSlotsUseCase {
    pub service_id: ID,
    pub start_date: String,
    pub end_date: String,
    pub timezone: Option<Tz>,
    pub duration: i64,
    pub interval: i64,
    pub host_user_ids: Option<Vec<ID>>,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::InvalidDate(msg) => {
                Self::BadClientData(format!(
                    "Invalid datetime: {}. Should be YYYY-MM-DD, e.g. January 1. 2020 => 2020-1-1",
                    msg
                ))
            }
            UseCaseError::InvalidInterval => {
                Self::BadClientData(
                    "Invalid interval specified. It should be between 10 - 60 minutes inclusively and be specified as milliseconds.".into()
                )
            }
            UseCaseError::InvalidTimespan => {
                Self::BadClientData("The provided start and end is invalid".into())
            }
            UseCaseError::ServiceNotFound => Self::NotFound("Service was not found".into())
        }
    }
}

#[derive(Debug)]
pub(crate) struct UseCaseRes {
    pub booking_slots: ServiceBookingSlots,
    pub service: ServiceWithUsers,
}

#[derive(Debug)]
pub(crate) enum UseCaseError {
    InternalError,
    ServiceNotFound,
    InvalidInterval,
    InvalidTimespan,
    InvalidDate(String),
}

impl From<BookingQueryError> for UseCaseError {
    fn from(e: BookingQueryError) -> Self {
        match e {
            BookingQueryError::InvalidInterval => Self::InvalidInterval,
            BookingQueryError::InvalidTimespan => Self::InvalidTimespan,
            BookingQueryError::InvalidDate(d) => Self::InvalidDate(d),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetServiceBookingSlotsUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "GetServiceBookingSlots";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        if !validate_slots_interval(self.interval) {
            return Err(UseCaseError::InvalidInterval);
        }

        let query = BookingSlotsQuery {
            start_date: self.start_date.clone(),
            end_date: self.end_date.clone(),
            timezone: self.timezone,
            interval: self.interval,
            duration: self.duration,
        };
        let booking_timespan = validate_bookingslots_query(&query)?;

        let service = match ctx.repos.services.find_with_users(&self.service_id).await {
            Ok(Some(s)) => s,
            Ok(None) => return Err(UseCaseError::ServiceNotFound),
            Err(_) => return Err(UseCaseError::InternalError),
        };

        if ServiceMultiPersonOptions::Group(0) == service.multi_person {
            return Ok(UseCaseRes {
                booking_slots: ServiceBookingSlots { dates: Vec::new() },
                service,
            });
        }

        let mut usecase_futures: Vec<_> = Vec::with_capacity(service.users.len());

        let timespan = TimeSpan::new(booking_timespan.start_time, booking_timespan.end_time);
        if timespan.greater_than(ctx.config.booking_slots_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        match &self.host_user_ids {
            Some(host_user_ids) => {
                for user_id in host_user_ids {
                    let user = service.users.iter().find(|u| u.user_id == *user_id);
                    if let Some(user) = user {
                        let timespan = timespan.clone();
                        usecase_futures.push(self.get_bookable_times(user, timespan, ctx));
                    }
                }
            }
            None => {
                for user in &service.users {
                    let timespan = timespan.clone();
                    usecase_futures.push(self.get_bookable_times(user, timespan, ctx));
                }
            }
        }

        let users_free_events: Vec<Result<UserFreeEvents, anyhow::Error>> =
            join_all(usecase_futures).await.into_iter().collect();

        // Handle errors in fetching events
        let mut all_users_free_events = Vec::new();
        for users_free_event in users_free_events {
            match users_free_event {
                Ok(events) => all_users_free_events.push(events),
                Err(_) => return Err(UseCaseError::InternalError),
            }
        }

        let mut booking_slots = get_service_bookingslots(
            all_users_free_events,
            &BookingSlotsOptions {
                interval: self.interval,
                duration: self.duration,
                end_time: booking_timespan.end_time,
                start_time: booking_timespan.start_time,
            },
        );

        booking_slots = match service.multi_person {
            ServiceMultiPersonOptions::Collective | ServiceMultiPersonOptions::Group(_) => {
                booking_slots
                    .into_iter()
                    .filter(|slot| slot.user_ids.len() == service.users.len())
                    .collect()
            }
            _ => booking_slots,
        };

        Ok(UseCaseRes {
            booking_slots: ServiceBookingSlots::new(booking_slots, chrono_tz::UTC),
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
        ctx: &NitteiContext,
    ) -> anyhow::Result<CompatibleInstances> {
        let empty = CompatibleInstances::new(Vec::new());
        match &user.availability {
            TimePlan::Calendar(id) => {
                let calendar = match user_calendars.iter().find(|cal| cal.id == *id) {
                    Some(cal) => cal,
                    None => {
                        return Ok(empty);
                    }
                };
                let all_calendar_events = ctx
                    .repos
                    .events
                    .find_by_calendar(id, Some(timespan))
                    .await
                    .unwrap_or_default();

                let all_event_instances = all_calendar_events
                    .iter()
                    .flat_map(|e| e.expand(Some(timespan), &calendar.settings))
                    .collect::<Vec<_>>();

                Ok(get_free_busy(all_event_instances).free)
            }
            TimePlan::Schedule(id) => match ctx.repos.schedules.find(id).await {
                Ok(Some(schedule)) if schedule.user_id == user.user_id => {
                    Ok(schedule.freebusy(timespan))
                }
                Ok(_) => Ok(empty),
                Err(_) => anyhow::bail!("Unable to fetch user schedule"),
            },
            TimePlan::Empty => Ok(empty),
        }
    }

    async fn get_user_busy(
        &self,
        user: &ServiceResource,
        user_nittei_calendars: &[Calendar],
        timespan: &TimeSpan,
        ctx: &NitteiContext,
    ) -> anyhow::Result<CompatibleInstances> {
        let busy_calendars = match ctx
            .repos
            .service_user_busy_calendars
            .find(&user.service_id, &user.user_id)
            .await
        {
            Ok(val) => val,
            Err(_) => return Ok(CompatibleInstances::new(Vec::new())),
        };

        let nittei_busy_calendar_ids = busy_calendars
            .iter()
            .filter_map(|bc| match bc {
                BusyCalendar::Nittei(id) => Some(id),
                _ => None,
            })
            .collect::<Vec<_>>();
        let nittei_busy_calendars = user_nittei_calendars
            .iter()
            .filter(|c| nittei_busy_calendar_ids.contains(&&c.id))
            .collect::<Vec<_>>();
        let google_busy_calendar_ids = busy_calendars
            .iter()
            .filter_map(|bc| match bc {
                BusyCalendar::Google(id) => Some(id.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let outlook_busy_calendar_ids = busy_calendars
            .iter()
            .filter_map(|bc| match bc {
                BusyCalendar::Outlook(id) => Some(id.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut busy_events: Vec<EventInstance> = Vec::new();

        let all_service_resources = ctx.repos.service_users.find_by_user(&user.user_id).await?;

        let mut busy_service_events = ctx
            .repos
            .events
            .find_user_service_events(&user.user_id, false, timespan.start(), timespan.end())
            .await?
            .into_iter()
            // Assuming all events of this type dont have a recurrence rule
            .filter(|e| match &e.service_id {
                Some(service_id) => service_id != &self.service_id,
                _ => unreachable!("Queried only for events with a service id"),
            })
            .map(|e| EventInstance {
                busy: true,
                start_time: e.start_time,
                end_time: e.end_time,
            })
            .collect::<Vec<_>>();
        busy_events.append(&mut busy_service_events);

        for cal in nittei_busy_calendars {
            match ctx
                .repos
                .events
                .find_by_calendar(&cal.id, Some(timespan))
                .await
            {
                Ok(calendar_events) => {
                    let mut calendar_busy_events = calendar_events
                        .into_iter()
                        .filter(|e| e.busy)
                        .flat_map(|e| {
                            let mut instances = e.expand(Some(timespan), &cal.settings);

                            // Add buffer to instances if event is a service event
                            if let Some(service_id) = e.service_id {
                                if let Some(service_resource) = all_service_resources
                                    .iter()
                                    .find(|s| s.service_id == service_id)
                                {
                                    let buffer_after_in_millis = TimeDelta::milliseconds(
                                        service_resource.buffer_after * 60 * 1000,
                                    );
                                    let buffer_before_in_millis = TimeDelta::milliseconds(
                                        service_resource.buffer_before * 60 * 1000,
                                    );
                                    for instance in instances.iter_mut() {
                                        instance.end_time += buffer_after_in_millis;
                                        instance.start_time -= buffer_before_in_millis;
                                    }
                                }
                            }
                            instances
                        })
                        .collect::<Vec<_>>();

                    busy_events.append(&mut calendar_busy_events);
                }
                Err(e) => {
                    error!("Unable to fetch user calendars: {}", e);
                }
            }
        }

        if !google_busy_calendar_ids.is_empty() {
            let user = if let Some(user) = ctx.repos.users.find(&user.user_id).await? {
                user
            } else {
                warn!("User not found: {}", user.user_id);
                return Ok(CompatibleInstances::new(Vec::new()));
            };
            match GoogleCalendarProvider::new(&user, ctx).await {
                Ok(google_calendar_provider) => {
                    let query = FreeBusyProviderQuery {
                        calendar_ids: google_busy_calendar_ids,
                        end: timespan.end(),
                        start: timespan.start(),
                    };
                    let google_busy = google_calendar_provider.freebusy(query).await;
                    for google_busy_event in google_busy.inner() {
                        busy_events.push(google_busy_event);
                    }
                }
                Err(_) => {
                    error!(
                        "Unable to initialize google calendar provider for user : {:?}",
                        user
                    );
                }
            }
        }

        if !outlook_busy_calendar_ids.is_empty() {
            let user = if let Some(user) = ctx.repos.users.find(&user.user_id).await? {
                user
            } else {
                warn!("User not found: {}", user.user_id);
                return Ok(CompatibleInstances::new(Vec::new()));
            };
            if let Ok(provider) = OutlookCalendarProvider::new(&user, ctx).await {
                let query = FreeBusyProviderQuery {
                    calendar_ids: outlook_busy_calendar_ids,
                    end: timespan.end(),
                    start: timespan.start(),
                };
                let outlook_busy = provider.freebusy(query).await;
                for outlook_busy_event in outlook_busy.inner() {
                    busy_events.push(outlook_busy_event);
                }
            }
        }

        // This should be optimized later
        Ok(CompatibleInstances::new(busy_events))
    }

    /// Ensure that calendar timespan fits within user settings for when
    /// it should be bookable
    fn parse_calendar_timespan(
        user: &ServiceResource,
        mut timespan: TimeSpan,
        ctx: &NitteiContext,
    ) -> Result<TimeSpan, ()> {
        let first_available = ctx.sys.get_timestamp()
            + TimeDelta::milliseconds(user.closest_booking_time * 60 * 1000);
        if timespan.start() < first_available {
            timespan = TimeSpan::new(first_available, timespan.end());
        }
        if let Some(furthest_booking_time) = user.furthest_booking_time {
            let last_available = ctx.sys.get_timestamp()
                + TimeDelta::milliseconds(furthest_booking_time * 60 * 1000);
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
        ctx: &NitteiContext,
    ) -> anyhow::Result<UserFreeEvents> {
        let empty = UserFreeEvents {
            free_events: CompatibleInstances::new(Vec::new()),
            user_id: service_resource.user_id.clone(),
        };

        match Self::parse_calendar_timespan(service_resource, timespan, ctx) {
            Ok(parsed_timespan) => timespan = parsed_timespan,
            Err(_) => return Ok(empty),
        }

        let user_calendars = ctx
            .repos
            .calendars
            .find_by_user(&service_resource.user_id)
            .await?;
        // let busy_calendars = user_calendars
        //     .iter()
        //     .filter(|cal| {
        //         service_resource
        //             .busy
        //             .iter()
        //             .find(|busy_calendar| match busy_calendar {
        //                 BusyCalendar::nittei(id) if *id == cal.id => true,
        //                 _ => false,
        //             })
        //             .is_some()
        //     })
        //     .collect::<Vec<_>>();

        let mut free_events = self
            .get_user_availability(service_resource, &user_calendars, &timespan, ctx)
            .await?;

        let busy_events = self
            .get_user_busy(service_resource, &user_calendars, &timespan, ctx)
            .await?;

        free_events.remove_instances(&busy_events, 0);

        Ok(UserFreeEvents {
            free_events,
            user_id: service_resource.user_id.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use chrono::{prelude::*, Utc};
    use nittei_domain::{
        Account,
        Calendar,
        CalendarEvent,
        RRuleOptions,
        Service,
        ServiceResource,
        User,
    };
    use nittei_infra::{setup_context, ISys};

    use super::*;

    struct TestContext {
        ctx: NitteiContext,
        service: Service,
        account: Account,
    }

    struct DummySys {}

    impl ISys for DummySys {
        fn get_timestamp_millis(&self) -> i64 {
            0
        }
        fn get_timestamp(&self) -> DateTime<Utc> {
            Utc.timestamp_millis_opt(0).unwrap()
        }
    }

    async fn setup() -> TestContext {
        let mut ctx = setup_context().await.unwrap();
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

    async fn setup_service_users(ctx: &NitteiContext, service: &mut Service, account_id: &ID) {
        let user1 = User::new(account_id.clone(), None);
        let user2 = User::new(account_id.clone(), None);
        ctx.repos.users.insert(&user1).await.unwrap();
        ctx.repos.users.insert(&user2).await.unwrap();
        let mut resource1 = ServiceResource {
            user_id: user1.id.clone(),
            service_id: service.id.clone(),
            buffer_after: 0,
            buffer_before: 0,
            availability: TimePlan::Empty,
            closest_booking_time: 0,
            furthest_booking_time: None,
        };
        let mut resource2 = ServiceResource {
            user_id: user2.id.clone(),
            service_id: service.id.clone(),
            buffer_after: 0,
            buffer_before: 0,
            availability: TimePlan::Empty,
            closest_booking_time: 0,
            furthest_booking_time: None,
        };

        let calendar_user_1 = Calendar::new(&resource1.user_id, account_id);
        resource1.availability = TimePlan::Calendar(calendar_user_1.id.clone());
        let calendar_user_2 = Calendar::new(&resource2.user_id, account_id);
        resource2.availability = TimePlan::Calendar(calendar_user_2.id.clone());

        ctx.repos.calendars.insert(&calendar_user_1).await.unwrap();
        ctx.repos.calendars.insert(&calendar_user_2).await.unwrap();

        let availability_event1 = CalendarEvent {
            account_id: account_id.clone(),
            calendar_id: calendar_user_1.id,
            duration: 1000 * 60 * 60,
            start_time: DateTime::from_timestamp_millis(1000 * 60 * 60).unwrap(),
            user_id: resource1.user_id.to_owned(),
            ..Default::default()
        };
        let availability_event2 = CalendarEvent {
            account_id: account_id.clone(),
            calendar_id: calendar_user_2.id.clone(),
            duration: 1000 * 60 * 60,
            start_time: DateTime::from_timestamp_millis(1000 * 60 * 60).unwrap(),
            user_id: resource2.user_id.to_owned(),
            ..Default::default()
        };
        let mut availability_event3 = CalendarEvent {
            account_id: account_id.clone(),
            calendar_id: calendar_user_2.id,
            duration: 1000 * 60 * 105,
            start_time: DateTime::from_timestamp_millis(1000 * 60 * 60 * 4).unwrap(),
            user_id: resource2.user_id.to_owned(),
            ..Default::default()
        };
        let recurrence = RRuleOptions::default();
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
            timezone: Some(chrono_tz::UTC),
            interval: 1000 * 60 * 15,
            service_id: service.id,
            host_user_ids: None,
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
            timezone: Some(chrono_tz::UTC),
            interval: 1000 * 60 * 15,
            service_id: service.id.clone(),
            host_user_ids: None,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let mut booking_slots = res.unwrap().booking_slots;
        assert_eq!(booking_slots.dates.len(), 1);
        let booking_slots = booking_slots.dates.remove(0).slots;

        assert_eq!(booking_slots.len(), 4);
        for (i, booking_slot) in booking_slots.iter().enumerate().take(4) {
            assert_eq!(booking_slot.duration, usecase.duration);
            assert_eq!(booking_slot.user_ids.len(), 1);
            assert_eq!(
                booking_slot.start,
                Utc.with_ymd_and_hms(2010, 1, 1, 4, 15 * i as u32, 0)
                    .unwrap()
            );
        }

        let mut usecase = GetServiceBookingSlotsUseCase {
            start_date: "1970-1-1".into(),
            end_date: "1970-1-1".into(),
            duration: 1000 * 60 * 60,
            timezone: Some(chrono_tz::UTC),
            interval: 1000 * 60 * 15,
            service_id: service.id,
            host_user_ids: None,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let mut booking_slots = res.unwrap().booking_slots;
        assert_eq!(booking_slots.dates.len(), 1);
        let booking_slots = booking_slots.dates.remove(0).slots;

        assert_eq!(booking_slots.len(), 5);
        assert_eq!(booking_slots[0].user_ids.len(), 2);
        for (i, booking_slot) in booking_slots.iter().enumerate().take(5) {
            assert_eq!(booking_slot.duration, usecase.duration);
            if i > 0 {
                assert_eq!(booking_slot.user_ids.len(), 1);
                assert_eq!(
                    booking_slot.start,
                    Utc.with_ymd_and_hms(1970, 1, 1, 4, 15 * (i - 1) as u32, 0)
                        .unwrap()
                );
            }
        }
    }
}
