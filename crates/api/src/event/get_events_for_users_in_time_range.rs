use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_validator::Json;
use chrono::{DateTime, Utc};
use nittei_api_structs::get_events_for_users_in_time_range::*;
use nittei_domain::{
    EventWithInstances,
    ID,
    TimeSpan,
    expand_event_and_remove_exceptions,
    generate_map_exceptions_original_start_times,
};
use nittei_infra::NitteiContext;
use tracing::error;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

/// Get events for users in a time range
///
/// Optionally, it can generate the instances of the recurring events
pub async fn get_events_for_users_in_time_range_controller(
    http_req: HttpRequest,
    ctx: web::Data<NitteiContext>,
    Json(body): Json<RequestBody>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let usecase = GetEventsForUsersInTimeRangeUseCase {
        account_id: account.id,
        user_ids: body.user_ids.clone(),
        start_time: body.start_time,
        end_time: body.end_time,
        generate_instances: body.generate_instances.unwrap_or(false),
    };

    execute(usecase, &ctx)
        .await
        .map(|events| HttpResponse::Ok().json(APIResponse::new(events.events)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetEventsForUsersInTimeRangeUseCase {
    pub account_id: ID,
    pub user_ids: Vec<ID>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub generate_instances: bool,
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

#[async_trait::async_trait]
impl UseCase for GetEventsForUsersInTimeRangeUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEventsForUsersInTimeRange";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<UseCaseResponse, UseCaseError> {
        let timespan = TimeSpan::new(self.start_time, self.end_time);
        if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        let calendars = if self.generate_instances {
            ctx.repos
                .calendars
                .find_for_users(&self.user_ids)
                .await
                .map_err(|_| UseCaseError::InternalError)?
        } else {
            vec![]
        };

        let calendars_map = calendars
            .iter()
            .map(|calendar| (calendar.id.clone(), calendar))
            .collect::<std::collections::HashMap<_, _>>();

        // Get the events for the calendars
        let events = ctx
            .repos
            .events
            .find_busy_events_and_recurring_events_for_users(&self.user_ids, timespan.clone())
            .await
            .map_err(|err| {
                error!("Got an error while finding events {:?}", err);
                UseCaseError::InternalError
            })?
            .into_iter()
            .filter(|event| event.account_id == self.account_id)
            .collect::<Vec<_>>();

        // Create a map of recurrence_id to events (exceptions)
        // This is used to remove exceptions from the expanded events
        let map_recurring_event_id_to_exceptions =
            generate_map_exceptions_original_start_times(&events);

        let events = if !self.generate_instances {
            // If we don't want to generate instances, we just return the events
            events
                .into_iter()
                .map(|event| EventWithInstances {
                    event,
                    instances: vec![],
                })
                .collect::<Vec<_>>()
        } else {
            // For each event, expand it and keep the instances next to the event
            events
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

                    let instances =
                        expand_event_and_remove_exceptions(calendar, &event, exceptions, timespan)
                            .map_err(|e| {
                                error!("Got an error while expanding an event {:?}", e);
                                UseCaseError::InternalError
                            })?;

                    Ok(EventWithInstances { event, instances })
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .collect::<Vec<_>>()
        };

        Ok(UseCaseResponse { events })
    }
}
