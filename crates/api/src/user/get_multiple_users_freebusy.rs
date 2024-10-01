use std::collections::{HashMap, VecDeque};

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use futures::{future::join_all, stream, StreamExt};
use nittei_api_structs::multiple_freebusy::{APIResponse, RequestBody};
use nittei_domain::{Calendar, CalendarEvent, EventInstance, TimeSpan, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_public_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn get_multiple_freebusy_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let _account = protect_public_account_route(&http_req, &ctx).await?;

    let usecase = GetMultipleFreeBusyUseCase {
        user_ids: body.user_ids.clone(),
        start_time: body.start_time,
        end_time: body.end_time,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse(usecase_res.0)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetMultipleFreeBusyUseCase {
    pub user_ids: Vec<ID>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct GetMultipleFreeBusyResponse(pub HashMap<ID, Vec<EventInstance>>);

#[derive(Debug)]
pub enum UseCaseError {
    InternalError,
    InvalidTimespan,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::InvalidTimespan => {
                Self::BadClientData("The provided start_ts and end_ts is invalid".into())
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetMultipleFreeBusyUseCase {
    type Response = GetMultipleFreeBusyResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetMultipleFreebusy";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let timespan = TimeSpan::new(self.start_time, self.end_time);
        if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        // TODO: Implement this
        // if self.user_ids.is_empty() {
        //     return Err(UseCaseError::InvalidTimespan);
        // }

        let calendars = self
            .get_calendars_from_user_ids(ctx)
            .await
            .map_err(|_| UseCaseError::InternalError)?;

        let busy_event_instances = self
            .get_event_instances_from_calendars(&timespan, ctx, calendars)
            .await?;

        Ok(GetMultipleFreeBusyResponse(busy_event_instances))
    }
}

impl GetMultipleFreeBusyUseCase {
    async fn get_calendars_from_user_ids(
        &self,
        ctx: &NitteiContext,
    ) -> anyhow::Result<Vec<Calendar>> {
        let calendars: Vec<anyhow::Result<Vec<Calendar>>> = join_all(
            self.user_ids
                .iter()
                .map(|user_id| ctx.repos.calendars.find_by_user(user_id)),
        )
        .await
        .into_iter()
        .collect();

        let mut all_calendars = Vec::new();
        for res in calendars {
            match res {
                Ok(cals) => all_calendars.extend(cals),
                Err(_) => return Err(anyhow::anyhow!("Internal error")),
            }
        }

        Ok(all_calendars)
    }

    async fn get_event_instances_from_calendars(
        &self,
        timespan: &TimeSpan,
        ctx: &NitteiContext,
        calendars: Vec<Calendar>,
    ) -> Result<HashMap<ID, Vec<EventInstance>>, UseCaseError> {
        // For quick lookup by calendar id
        let calendars_lookup = calendars
            .iter()
            .map(|cal| (cal.id.to_string(), cal))
            .collect::<HashMap<_, _>>();

        // End result
        let mut events_per_user: HashMap<ID, Vec<EventInstance>> = HashMap::new();

        // Fetch all events for all calendars
        // This is not executed yet (lazy)
        let all_events_futures = calendars.iter().map(|calendar| async move {
            let events = ctx
                .repos
                .events
                .find_by_calendar(&calendar.id, Some(timespan))
                .await
                .unwrap_or_default(); // TODO: Handle error
            Ok((calendar.user_id.clone(), events)) as Result<(ID, Vec<CalendarEvent>), UseCaseError>
        });

        // Fetch events in chunks of 5
        let mut futures_stream = stream::iter(all_events_futures).chunks(5);

        // Fetch events in parallel (actual execution)
        while let Some(futures) = futures_stream.next().await {
            let events_res = join_all(futures).await;

            for event_result in events_res {
                match event_result {
                    Ok((user_id, events)) => {
                        let expanded_events =
                            self.expand_events(events, timespan, &calendars_lookup)?;
                        events_per_user.insert(user_id, expanded_events.into());
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(events_per_user)
    }

    fn expand_events(
        &self,
        events: Vec<CalendarEvent>,
        timespan: &TimeSpan,
        calendars_lookup: &HashMap<String, &Calendar>,
    ) -> Result<VecDeque<EventInstance>, UseCaseError> {
        let mut instances = VecDeque::new();
        for event in &events {
            let calendar = calendars_lookup
                .get(&event.calendar_id.to_string())
                .ok_or(UseCaseError::InternalError)?;

            let expanded_events = event.expand(Some(timespan), &calendar.settings);

            instances.extend(expanded_events);
        }
        Ok(instances)
    }
}

#[cfg(test)]
mod test {
    use nittei_domain::{Account, Calendar, CalendarEvent, Entity, RRuleOptions, User};
    use nittei_infra::setup_context;

    use super::*;

    #[actix_web::main]
    #[test]
    async fn multiple_freebusy_works() {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id(), &user.account_id, None, None);
        ctx.repos.calendars.insert(&calendar).await.unwrap();
        let one_hour = 1000 * 60 * 60;
        let mut e1 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            duration: one_hour,
            end_time: DateTime::<Utc>::MAX_UTC,
            ..Default::default()
        };
        let e1rr = RRuleOptions {
            count: Some(100),
            ..Default::default()
        };
        e1.set_recurrence(e1rr, &calendar.settings, true);

        let mut e2 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            duration: one_hour,
            end_time: DateTime::<Utc>::MAX_UTC,
            start_time: DateTime::from_timestamp_millis(one_hour * 4).unwrap(),
            ..Default::default()
        };
        let e2rr = RRuleOptions {
            count: Some(100),
            ..Default::default()
        };
        e2.set_recurrence(e2rr, &calendar.settings, true);

        let mut e3 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            duration: one_hour,
            end_time: DateTime::from_timestamp_millis(one_hour).unwrap(),
            ..Default::default()
        };
        let e3rr = RRuleOptions {
            count: Some(100),
            interval: 2,
            ..Default::default()
        };
        e3.set_recurrence(e3rr, &calendar.settings, true);

        ctx.repos.events.insert(&e1).await.unwrap();
        ctx.repos.events.insert(&e2).await.unwrap();
        ctx.repos.events.insert(&e3).await.unwrap();

        let mut usecase = GetMultipleFreeBusyUseCase {
            user_ids: vec![user.id().clone()],
            start_time: DateTime::from_timestamp_millis(86400000).unwrap(),
            end_time: DateTime::from_timestamp_millis(172800000).unwrap(),
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let map_instances = res.unwrap().0;
        assert_eq!(map_instances.len(), 1);
        assert!(map_instances.contains_key(&user.id()));
        let instances = map_instances.get(&user.id()).unwrap();
        assert_eq!(instances.len(), 2);
        assert_eq!(
            instances[0],
            EventInstance {
                busy: true,
                start_time: DateTime::from_timestamp_millis(86400000).unwrap(),
                end_time: DateTime::from_timestamp_millis(90000000).unwrap(),
            }
        );
        assert_eq!(
            instances[1],
            EventInstance {
                busy: true,
                start_time: DateTime::from_timestamp_millis(100800000).unwrap(),
                end_time: DateTime::from_timestamp_millis(104400000).unwrap(),
            }
        );
    }
}
