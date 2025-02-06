use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use nittei_api_structs::get_events_by_calendars::*;
use nittei_domain::{
    expand_event_and_remove_exceptions,
    generate_map_exceptions_original_start_times,
    EventWithInstances,
    TimeSpan,
    ID,
};
use nittei_infra::NitteiContext;
use tracing::error;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn get_events_by_calendars_controller(
    http_req: HttpRequest,
    query: web::Query<QueryParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
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
        .map_err(NitteiError::from)
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
    InternalError,
    NotFound(String, String),
    InvalidTimespan,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
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

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<UseCaseResponse, UseCaseError> {
        let calendars = ctx
            .repos
            .calendars
            .find_multiple(self.calendar_ids.iter().collect::<Vec<_>>())
            .await
            .map_err(|_| UseCaseError::InternalError)?;

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
                // Create a map of recurrence_id to events (exceptions)
                // This is used to remove exceptions from the expanded events
                let map_recurring_event_id_to_exceptions =
                    generate_map_exceptions_original_start_times(&events);

                // For each event, expand it and keep the instances next to the event
                let events = events
                    .into_iter()
                    .map(|event| {
                        let calendar = calendars_map.get(&event.calendar_id).ok_or_else(|| {
                            UseCaseError::NotFound(
                                "Calendar".to_string(),
                                event.calendar_id.to_string(),
                            )
                        })?;

                        // Get the exceptions of the event
                        let exceptions = map_recurring_event_id_to_exceptions
                            .get(&event.id)
                            .map(Vec::as_slice)
                            .unwrap_or(&[]);

                        // Expand the event and remove the exceptions
                        let instances = expand_event_and_remove_exceptions(
                            calendar, &event, exceptions, &timespan,
                        )
                        .map_err(|e| {
                            error!("Got an error while expanding an event {:?}", e);
                            UseCaseError::InternalError
                        })?;

                        Ok(EventWithInstances { event, instances })
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    // // Also it is possible that there are no instances in the expanded event, should remove them
                    .filter(|data| !data.instances.is_empty())
                    .collect::<Vec<_>>();

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
