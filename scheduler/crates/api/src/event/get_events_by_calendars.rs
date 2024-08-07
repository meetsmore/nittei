use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use nettu_scheduler_api_structs::get_events_by_calendars::*;
use nettu_scheduler_domain::{EventWithInstances, TimeSpan, ID};
use nettu_scheduler_infra::NettuContext;

use crate::{
    error::NettuError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn get_events_by_calendars_controller(
    http_req: HttpRequest,
    query: web::Query<QueryParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let calendar_ids = match &query.calendar_ids {
        Some(ids) => ids.clone(),
        None => vec![],
    };

    let usecase = GetEventsByCalendarsUseCase {
        account_id: account.id,
        calendar_ids,
        start_time: query.start_time,
        end_time: query.end_time,
    };

    execute(usecase, &ctx)
        .await
        .map(|events| HttpResponse::Ok().json(APIResponse::new(events.events)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
pub struct GetEventsByCalendarsUseCase {
    pub account_id: ID,
    pub calendar_ids: Vec<ID>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct UseCaseResponse {
    pub events: Vec<EventWithInstances>,
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(String, String),
    InvalidTimespan,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InvalidTimespan => {
                Self::BadClientData("The provided start_ts and end_ts is invalid".into())
            }
            UseCaseError::NotFound(entity, event_id) => Self::NotFound(format!(
                "The {} with id: {}, was not found.",
                entity, event_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventsByCalendarsUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEventsByCalendars";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<UseCaseResponse, UseCaseError> {
        let calendars = ctx
            .repos
            .calendars
            .find_multiple(self.calendar_ids.iter().collect::<Vec<_>>())
            .await;

        // Check that all calendars exist and belong to the same account
        if calendars.is_empty()
            || calendars.len() != self.calendar_ids.len()
            || !calendars
                .iter()
                .all(|cal| cal.account_id == self.account_id)
        {
            return Err(UseCaseError::NotFound(
                "Calendars not found".to_string(),
                self.calendar_ids
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            ));
        }

        let calendars_map = calendars
            .into_iter()
            .map(|cal| (cal.id.clone(), cal))
            .collect::<std::collections::HashMap<_, _>>();

        let timespan = TimeSpan::new(self.start_time, self.end_time);
        if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        let res = ctx
            .repos
            .events
            .find_by_calendars(self.calendar_ids.clone(), &timespan)
            .await;
        match res {
            Ok(events) => {
                let events = events
                    .into_iter()
                    .map(|event| {
                        let calendar = &calendars_map[&event.calendar_id];
                        let instances = event.expand(Some(&timespan), &calendar.settings);
                        EventWithInstances { event, instances }
                    })
                    // Also it is possible that there are no instances in the expanded event, should remove them
                    .filter(|data| !data.instances.is_empty())
                    .collect();

                Ok(UseCaseResponse { events })
            }
            _ => Err(UseCaseError::NotFound(
                "Events not found".to_string(),
                self.calendar_ids
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            )),
        }
    }
}
