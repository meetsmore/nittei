use std::collections::HashMap;

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use futures::{future::join_all, stream, StreamExt};
use nettu_scheduler_api_structs::multiple_freebusy::{APIResponse, RequestBody};
use nettu_scheduler_domain::{Calendar, CompatibleInstances, EventInstance, TimeSpan, ID};
use nettu_scheduler_infra::NettuContext;

use crate::{
    error::NettuError,
    shared::{
        auth::protect_public_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn get_multiple_freebusy_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let _account = protect_public_account_route(&http_req, &ctx).await?;

    let usecase = GetMultipleFreeBusyUseCase {
        user_ids: body.user_ids.clone(),
        start_time: body.start_time,
        end_time: body.end_time,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse {
                busy: usecase_res.busy.inner(),
            })
        })
        .map_err(NettuError::from)
}

#[derive(Debug)]
pub struct GetMultipleFreeBusyUseCase {
    pub user_ids: Vec<ID>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct GetMultipleFreeBusyResponse {
    pub busy: CompatibleInstances,
}

#[derive(Debug)]
pub enum UseCaseError {
    InvalidTimespan,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
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

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let timespan = TimeSpan::new(self.start_time, self.end_time);
        if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        // TODO: Implement this
        // if self.user_ids.is_empty() {
        //     return Err(UseCaseError::InvalidTimespan);
        // }

        let calendars = self.get_calendars_from_user_ids(ctx).await;

        let busy_event_instances = self
            .get_event_instances_from_calendars(&timespan, ctx, calendars)
            .await
            .into_iter()
            .filter(|e| e.busy)
            .collect::<Vec<_>>();

        let busy = CompatibleInstances::new(busy_event_instances);

        Ok(GetMultipleFreeBusyResponse { busy })
    }
}

impl GetMultipleFreeBusyUseCase {
    async fn get_calendars_from_user_ids(&self, ctx: &NettuContext) -> Vec<Calendar> {
        join_all(
            self.user_ids
                .iter()
                .map(|user_id| ctx.repos.calendars.find_by_user(user_id)),
        )
        .await
        .into_iter()
        .flatten()
        .collect()
    }

    async fn get_event_instances_from_calendars(
        &self,
        timespan: &TimeSpan,
        ctx: &NettuContext,
        calendars: Vec<Calendar>,
    ) -> Vec<EventInstance> {
        let all_events_futures = calendars.iter().map(|calendar| {
            ctx.repos
                .events
                .find_by_calendar(&calendar.id, Some(timespan))
        });

        let calendars_lookup = calendars
            .iter()
            .map(|cal| (cal.id.to_string(), cal))
            .collect::<HashMap<_, _>>();

        let mut all_events = Vec::new();

        // Fetch events in chunks of 5
        let mut futures_stream = stream::iter(all_events_futures).chunks(5);

        // Fetch events in parallel
        while let Some(futures) = futures_stream.next().await {
            let events_res = join_all(futures).await;
            let instances = events_res
                .into_iter()
                .map(|events_res| events_res.unwrap_or_default())
                .flat_map(|events| {
                    events
                        .into_iter()
                        .map(|event| {
                            let calendar = calendars_lookup
                                .get(&event.calendar_id.to_string())
                                .unwrap();
                            event.expand(Some(timespan), &calendar.settings)
                        })
                        // It is possible that there are no instances in the expanded event, should remove them
                        .filter(|instances| !instances.is_empty())
                })
                .flatten()
                .collect::<Vec<_>>();

            all_events.extend(instances);
        }

        all_events
    }
}

#[cfg(test)]
mod test {
    use nettu_scheduler_domain::{Account, Calendar, CalendarEvent, Entity, RRuleOptions, User};
    use nettu_scheduler_infra::setup_context;

    use super::*;

    #[actix_web::main]
    #[test]
    async fn multiple_freebusy_works() {
        let ctx = setup_context().await;
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id(), &user.account_id);
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
        let instances = res.unwrap().busy.inner();
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
