use std::collections::HashMap;

use actix_web::{web, HttpRequest, HttpResponse};
use futures::future::join_all;
use nettu_scheduler_api_structs::get_user_freebusy::{APIResponse, PathParams, QueryParams};
use nettu_scheduler_domain::{CompatibleInstances, EventInstance, TimeSpan, ID};
use nettu_scheduler_infra::NettuContext;

use crate::{
    error::NettuError,
    shared::{
        auth::protect_public_account_route,
        usecase::{execute, UseCase},
    },
};

/// "1,2,3" -> Vec<1,2,3>
pub fn parse_vec_query_value(val: &Option<String>) -> Option<Vec<ID>> {
    val.as_ref().map(|ids| {
        ids.split(',')
            .map(String::from)
            .flat_map(|id| id.parse::<ID>())
            .collect()
    })
}

pub async fn get_freebusy_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    mut params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let _account = protect_public_account_route(&http_req, &ctx).await?;

    let calendar_ids = parse_vec_query_value(&query_params.calendar_ids);

    let usecase = GetFreeBusyUseCase {
        user_id: std::mem::take(&mut params.user_id),
        calendar_ids,
        start_ts: query_params.start_ts,
        end_ts: query_params.end_ts,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse {
                busy: usecase_res.busy.inner(),
                user_id: usecase_res.user_id.to_string(),
            })
        })
        .map_err(NettuError::from)
}

#[derive(Debug)]
pub struct GetFreeBusyUseCase {
    pub user_id: ID,
    pub calendar_ids: Option<Vec<ID>>,
    pub start_ts: i64,
    pub end_ts: i64,
}

#[derive(Debug)]
pub struct GetFreeBusyResponse {
    pub busy: CompatibleInstances,
    pub user_id: ID,
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
impl UseCase for GetFreeBusyUseCase {
    type Response = GetFreeBusyResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetUserFreebusy";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let timespan = TimeSpan::new(self.start_ts, self.end_ts);
        if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        let busy_event_instances = self
            .get_event_instances_from_calendars(&timespan, ctx)
            .await
            .into_iter()
            .filter(|e| e.busy)
            .collect::<Vec<_>>();

        let busy = CompatibleInstances::new(busy_event_instances);

        Ok(GetFreeBusyResponse {
            busy,
            user_id: self.user_id.to_owned(),
        })
    }
}

impl GetFreeBusyUseCase {
    async fn get_event_instances_from_calendars(
        &self,
        timespan: &TimeSpan,
        ctx: &NettuContext,
    ) -> Vec<EventInstance> {
        let calendar_ids = match &self.calendar_ids {
            Some(ids) if !ids.is_empty() => ids,
            _ => return Vec::new(),
        };

        // can probably make query to event repo instead
        let mut calendars = ctx.repos.calendars.find_by_user(&self.user_id).await;

        if !calendar_ids.is_empty() {
            calendars.retain(|cal| calendar_ids.contains(&cal.id));
        }

        let calendars_lookup: HashMap<_, _> = calendars
            .iter()
            .map(|cal| (cal.id.to_string(), cal))
            .collect();

        let all_events_futures = calendars.iter().map(|calendar| {
            ctx.repos
                .events
                .find_by_calendar(&calendar.id, Some(timespan))
        });

        join_all(all_events_futures)
            .await
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
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod test {
    use nettu_scheduler_domain::{Account, Calendar, CalendarEvent, Entity, RRuleOptions, User};
    use nettu_scheduler_infra::setup_context;

    use super::*;

    #[test]
    fn it_parses_vec_query_params_correctly() {
        assert_eq!(parse_vec_query_value(&None), None);
        assert_eq!(
            parse_vec_query_value(&Some("".to_string())),
            Some(Vec::new())
        );
        assert_eq!(
            parse_vec_query_value(&Some("2".to_string())),
            Some(Vec::new())
        );
        let ids = vec![ID::default(), ID::default()];
        assert_eq!(
            parse_vec_query_value(&Some(format!("{},{}", ids[0], ids[1]))),
            Some(ids)
        );
    }

    #[actix_web::main]
    #[test]
    async fn freebusy_works() {
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
            end_ts: CalendarEvent::get_max_timestamp(),
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
            end_ts: CalendarEvent::get_max_timestamp(),
            start_ts: one_hour * 4,
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
            end_ts: one_hour,
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

        let mut usecase = GetFreeBusyUseCase {
            user_id: user.id().clone(),
            calendar_ids: Some(vec![calendar.id.clone()]),
            start_ts: 86400000,
            end_ts: 172800000,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let instances = res.unwrap().busy.inner();
        assert_eq!(instances.len(), 2);
        assert_eq!(
            instances[0],
            EventInstance {
                busy: true,
                start_ts: 86400000,
                end_ts: 90000000,
            }
        );
        assert_eq!(
            instances[1],
            EventInstance {
                busy: true,
                start_ts: 100800000,
                end_ts: 104400000,
            }
        );
    }
}
