use actix_web::{HttpRequest, HttpResponse, web};
use chrono::{DateTime, Utc};
use nittei_api_structs::get_calendar_events::{APIResponse, PathParams, QueryParams};
use nittei_domain::{
    Calendar,
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
        auth::{account_can_modify_calendar, protect_admin_route, protect_route},
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/user/calendar/{calendar_id}/events",
    summary = "Get events for a calendar (admin only)"
)]
pub async fn get_calendar_events_admin_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = GetCalendarEventsUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
        start_time: query_params.start_time,
        end_time: query_params.end_time,
    };

    execute(usecase, &ctx)
        .await
        .map_err(NitteiError::from)
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.calendar, usecase_res.events))
        })
}

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/calendar/{calendar_id}/events",
    summary = "Get events for a calendar"
)]
pub async fn get_calendar_events_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetCalendarEventsUseCase {
        user_id: user.id,
        calendar_id: path.calendar_id.clone(),
        start_time: query_params.start_time,
        end_time: query_params.end_time,
    };

    execute(usecase, &ctx)
        .await
        .map_err(NitteiError::from)
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.calendar, usecase_res.events))
        })
}
#[derive(Debug)]
pub struct GetCalendarEventsUseCase {
    pub calendar_id: ID,
    pub user_id: ID,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct UseCaseResponse {
    calendar: Calendar,
    events: Vec<EventWithInstances>,
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(ID),
    InvalidTimespan,
    IntervalServerError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InvalidTimespan => {
                Self::BadClientData("The start and end timespan is invalid".into())
            }
            UseCaseError::NotFound(calendar_id) => Self::NotFound(format!(
                "The calendar with id: {}, was not found.",
                calendar_id
            )),
            UseCaseError::IntervalServerError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for GetCalendarEventsUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetCalendarEvents";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let calendar = ctx
            .repos
            .calendars
            .find(&self.calendar_id)
            .await
            .map_err(|_| UseCaseError::IntervalServerError)?;

        let timespan = TimeSpan::new(self.start_time, self.end_time);
        if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => {
                // Get the calendar itself
                let calendar_events = ctx
                    .repos
                    .events
                    .find_by_calendar(&calendar.id, Some(timespan.clone()))
                    .await
                    .map_err(|e| {
                        error!("{:?}", e);
                        UseCaseError::IntervalServerError
                    })?;

                // Create a map of recurrence_id to events (exceptions)
                // This is used to remove exceptions from the expanded events
                let map_recurring_event_id_to_exceptions =
                    generate_map_exceptions_original_start_times(&calendar_events);

                // For each event, expand it and keep the instances next to the event
                let events = calendar_events
                    .into_iter()
                    .map(|event| {
                        // Get the exceptions for the event
                        let exceptions = map_recurring_event_id_to_exceptions
                            .get(&event.id)
                            .map(Vec::as_slice)
                            .unwrap_or(&[]);

                        let timespan = timespan.clone();
                        // Expand the event and remove the exceptions
                        let instances = expand_event_and_remove_exceptions(
                            &calendar, &event, exceptions, timespan,
                        )
                        .map_err(|e| {
                            error!("Got an error while expanding an event {:?}", e);
                            UseCaseError::IntervalServerError
                        })?;

                        Ok(EventWithInstances { event, instances })
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    // Also it is possible that there are no instances in the expanded event, should remove them
                    .filter(|data| !data.instances.is_empty())
                    .collect::<Vec<_>>();

                Ok(UseCaseResponse { calendar, events })
            }
            _ => Err(UseCaseError::NotFound(self.calendar_id.clone())),
        }
    }
}
