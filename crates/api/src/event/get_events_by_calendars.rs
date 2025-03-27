use actix_web::{HttpRequest, HttpResponse, web};
use chrono::{DateTime, Utc};
use nittei_api_structs::get_events_by_calendars::*;
use nittei_domain::{
    EventWithInstances,
    ID,
    TimeSpan,
    expand_event_and_remove_exceptions,
    generate_map_exceptions_original_start_times,
};
use nittei_infra::NitteiContext;
use nittei_utils::config::APP_CONFIG;
use tracing::error;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn get_events_by_calendars_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    query: web::Query<QueryParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let calendar_ids = match &query.calendar_ids {
        Some(ids) => ids.clone(),
        None => vec![],
    };

    let usecase = GetEventsByCalendarsUseCase {
        account_id: account.id,
        user_id: path_params.user_id.clone(),
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
    pub user_id: ID,
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
                Self::BadClientData("The provided start_ts and end_ts are invalid".into())
            }
            UseCaseError::NotFound(entity, event_id) => Self::NotFound(format!(
                "The {} with id: {}, was not found.",
                entity, event_id
            )),
        }
    }
}

#[async_trait::async_trait]
impl UseCase for GetEventsByCalendarsUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEventsByCalendars";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<UseCaseResponse, UseCaseError> {
        let calendars = if self.calendar_ids.is_empty() {
            // If no calendar_ids are provided, fetch all calendars for the user
            ctx.repos
                .calendars
                .find_by_user(&self.user_id)
                .await
                .map_err(|_| {
                    error!("Error while fetching calendars by user_id");
                    UseCaseError::InternalError
                })?
        } else {
            // Else, fetch the calendars by the provided calendar_ids
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

            calendars
        };

        // Create a map of calendar_id to calendar
        let calendars_map = calendars
            .into_iter()
            .map(|cal| (cal.id.clone(), cal))
            .collect::<std::collections::HashMap<_, _>>();

        let timespan = TimeSpan::new(self.start_time, self.end_time);
        if timespan.greater_than(APP_CONFIG.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        // Get the events for the calendars
        let res = ctx
            .repos
            .events
            .find_by_calendars(&self.calendar_ids, timespan.clone())
            .await;

        // If the events are found, expand them and remove the exceptions
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
                        let timespan = timespan.clone();
                        let instances = expand_event_and_remove_exceptions(
                            calendar, &event, exceptions, timespan,
                        )
                        .map_err(|e| {
                            error!("Got an error while expanding an event {:?}", e);
                            UseCaseError::InternalError
                        })?;

                        Ok(EventWithInstances { event, instances })
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    // Also it is possible that there are no instances in the expanded event, should remove them
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
