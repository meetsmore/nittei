use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, web};
use chrono::{DateTime, Utc};
use nittei_api_structs::get_user_freebusy::{APIResponse, PathParams, QueryParams};
use nittei_domain::{
    CompatibleInstances,
    EventInstance,
    ID,
    TimeSpan,
    expand_all_events_and_remove_exceptions,
};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_public_account_route,
        usecase::{UseCase, execute},
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
    mut query_params: web::Query<QueryParams>,
    mut params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let _account = protect_public_account_route(&http_req, &ctx).await?;

    let usecase = GetFreeBusyUseCase {
        user_id: std::mem::take(&mut params.user_id),
        calendar_ids: query_params.calendar_ids.take(),
        start_time: query_params.start_time,
        end_time: query_params.end_time,
        include_tentative: query_params.include_tentative,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse {
                busy: usecase_res.busy.inner().into(),
                user_id: usecase_res.user_id.to_string(),
            })
        })
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetFreeBusyUseCase {
    pub user_id: ID,
    pub calendar_ids: Option<Vec<ID>>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub include_tentative: Option<bool>,
}

#[derive(Debug)]
pub struct GetFreeBusyResponse {
    pub busy: CompatibleInstances,
    pub user_id: ID,
}

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
impl UseCase for GetFreeBusyUseCase {
    type Response = GetFreeBusyResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetUserFreebusy";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let timespan = TimeSpan::new(self.start_time, self.end_time);
        if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        let busy_event_instances = self
            .get_event_instances_from_calendars(&timespan, ctx)
            .await
            .map_err(|_| UseCaseError::InternalError)?
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
        ctx: &NitteiContext,
    ) -> anyhow::Result<Vec<EventInstance>> {
        // can probably make query to event repo instead
        let mut calendars = ctx.repos.calendars.find_by_user(&self.user_id).await?;

        let calendar_ids = match &self.calendar_ids {
            Some(ids) if !ids.is_empty() => ids.to_owned(),
            _ => calendars.iter().map(|cal| cal.id.clone()).collect(),
        };

        if !calendar_ids.is_empty() {
            calendars.retain(|cal| calendar_ids.contains(&cal.id));
        }

        let calendars_lookup: HashMap<_, _> = calendars
            .iter()
            .map(|cal| (cal.id.to_string(), cal))
            .collect();

        let events = ctx
            .repos
            .events
            .find_busy_events_and_recurring_events_for_calendars(
                &calendar_ids,
                timespan,
                self.include_tentative.unwrap_or(false),
            )
            .await?;

        // If we have no events, return early
        if events.is_empty() {
            return Ok(Vec::new());
        }

        // Expand the events, remove the exceptions and return the expanded events
        let mut events =
            expand_all_events_and_remove_exceptions(&calendars_lookup, &events, timespan)?;

        // Sort the events by start_time
        events.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        Ok(events)
    }
}

#[cfg(test)]
mod test {
    use nittei_domain::{Account, Calendar, CalendarEvent, Entity, RRuleOptions, User};
    use nittei_infra::setup_context;

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
    async fn test_freebusy_recurring() {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id(), &user.account_id, None, None);
        ctx.repos.calendars.insert(&calendar).await.unwrap();
        let one_hour = 1000 * 60 * 60;

        let daily_recurring_event = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            status: nittei_domain::CalendarEventStatus::Confirmed,
            start_time: DateTime::parse_from_rfc3339("2025-01-05T11:00:00Z")
                .unwrap()
                .to_utc(),
            end_time: DateTime::parse_from_rfc3339("2025-01-05T12:00:00Z")
                .unwrap()
                .to_utc(),
            duration: one_hour,
            recurrence: Some(RRuleOptions {
                // Everyday - infinitely
                freq: nittei_domain::RRuleFrequency::Daily,
                interval: 1,
                ..Default::default()
            }),
            ..Default::default()
        };
        ctx.repos
            .events
            .insert(&daily_recurring_event)
            .await
            .unwrap();

        let daily_recurring_event_finished = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            status: nittei_domain::CalendarEventStatus::Confirmed,
            start_time: DateTime::parse_from_rfc3339("2025-01-05T18:00:00Z")
                .unwrap()
                .to_utc(),
            end_time: DateTime::parse_from_rfc3339("2025-01-05T19:00:00Z")
                .unwrap()
                .to_utc(),
            duration: one_hour,
            recurrence: Some(RRuleOptions {
                // Everyday until 2025-01-08
                freq: nittei_domain::RRuleFrequency::Daily,
                interval: 1,
                until: Some(
                    DateTime::parse_from_rfc3339("2025-01-08T19:00:00Z")
                        .unwrap()
                        .to_utc(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        };
        ctx.repos
            .events
            .insert(&daily_recurring_event_finished)
            .await
            .unwrap();

        let weekly_recurring_event = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            status: nittei_domain::CalendarEventStatus::Confirmed,
            start_time: DateTime::parse_from_rfc3339("2025-01-09T14:00:00Z")
                .unwrap()
                .to_utc(),
            end_time: DateTime::parse_from_rfc3339("2025-01-09T15:00:00Z")
                .unwrap()
                .to_utc(),
            duration: one_hour,
            recurrence: Some(RRuleOptions {
                // Every week - infinitely
                freq: nittei_domain::RRuleFrequency::Weekly,
                interval: 1,
                ..Default::default()
            }),
            ..Default::default()
        };
        ctx.repos
            .events
            .insert(&weekly_recurring_event)
            .await
            .unwrap();

        let mut freebusy_params: GetFreeBusyUseCase = GetFreeBusyUseCase {
            user_id: user.id().clone(),
            calendar_ids: Some(vec![calendar.id.clone()]),
            start_time: DateTime::parse_from_rfc3339("2025-01-09T00:00:00Z")
                .unwrap()
                .to_utc(),
            end_time: DateTime::parse_from_rfc3339("2025-01-15T00:00:00Z")
                .unwrap()
                .to_utc(),
            include_tentative: None,
        };

        let freebusy = freebusy_params.execute(&ctx).await.unwrap();

        let dates = freebusy.busy.inner();

        assert_eq!(dates.len(), 7);

        // Expect 6 dates from the recurring daily (event 1)
        // And expect 1 date from the recurring weekly (event 2)
        assert_eq!(
            // Daily n1
            dates[0],
            EventInstance {
                busy: true,
                start_time: DateTime::parse_from_rfc3339("2025-01-09T11:00:00Z")
                    .unwrap()
                    .to_utc(),
                end_time: DateTime::parse_from_rfc3339("2025-01-09T12:00:00Z")
                    .unwrap()
                    .to_utc(),
            }
        );
        // Weekly n1
        assert_eq!(
            dates[1],
            EventInstance {
                busy: true,
                start_time: DateTime::parse_from_rfc3339("2025-01-09T14:00:00Z")
                    .unwrap()
                    .to_utc(),
                end_time: DateTime::parse_from_rfc3339("2025-01-09T15:00:00Z")
                    .unwrap()
                    .to_utc(),
            }
        );
        // Daily n2
        assert_eq!(
            dates[2],
            EventInstance {
                busy: true,
                start_time: DateTime::parse_from_rfc3339("2025-01-10T11:00:00Z")
                    .unwrap()
                    .to_utc(),
                end_time: DateTime::parse_from_rfc3339("2025-01-10T12:00:00Z")
                    .unwrap()
                    .to_utc(),
            }
        );
        // Daily n3
        assert_eq!(
            dates[3],
            EventInstance {
                busy: true,
                start_time: DateTime::parse_from_rfc3339("2025-01-11T11:00:00Z")
                    .unwrap()
                    .to_utc(),
                end_time: DateTime::parse_from_rfc3339("2025-01-11T12:00:00Z")
                    .unwrap()
                    .to_utc(),
            }
        );
        // Daily n4
        assert_eq!(
            dates[4],
            EventInstance {
                busy: true,
                start_time: DateTime::parse_from_rfc3339("2025-01-12T11:00:00Z")
                    .unwrap()
                    .to_utc(),
                end_time: DateTime::parse_from_rfc3339("2025-01-12T12:00:00Z")
                    .unwrap()
                    .to_utc(),
            }
        );
        // Daily n5
        assert_eq!(
            dates[5],
            EventInstance {
                busy: true,
                start_time: DateTime::parse_from_rfc3339("2025-01-13T11:00:00Z")
                    .unwrap()
                    .to_utc(),
                end_time: DateTime::parse_from_rfc3339("2025-01-13T12:00:00Z")
                    .unwrap()
                    .to_utc(),
            }
        );
        // Daily n6
        assert_eq!(
            dates[6],
            EventInstance {
                busy: true,
                start_time: DateTime::parse_from_rfc3339("2025-01-14T11:00:00Z")
                    .unwrap()
                    .to_utc(),
                end_time: DateTime::parse_from_rfc3339("2025-01-14T12:00:00Z")
                    .unwrap()
                    .to_utc(),
            }
        );
    }
}
