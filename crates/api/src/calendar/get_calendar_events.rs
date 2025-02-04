use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use nittei_api_structs::get_calendar_events::{APIResponse, PathParams, QueryParams};
use nittei_domain::{Calendar, EventWithInstances, TimeSpan, ID};
use nittei_infra::NitteiContext;
use tracing::error;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_calendar, protect_account_route, protect_route},
        usecase::{execute, UseCase},
    },
};

pub async fn get_calendar_events_admin_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
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

#[async_trait::async_trait(?Send)]
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
                let calendar_events = ctx
                    .repos
                    .events
                    .find_by_calendar(&calendar.id, Some(&timespan))
                    .await
                    .map_err(|e| {
                        error!("{:?}", e);
                        UseCaseError::IntervalServerError
                    })?;

                let events = calendar_events
                    .into_iter()
                    .map(|event| {
                        // Todo: handle error
                        let instances = event
                            .expand(Some(&timespan), &calendar.settings)
                            .unwrap_or_default();
                        EventWithInstances { event, instances }
                    })
                    // Also it is possible that there are no instances in the expanded event, should remove them
                    .filter(|data| !data.instances.is_empty())
                    .collect();

                Ok(UseCaseResponse { calendar, events })
            }
            _ => Err(UseCaseError::NotFound(self.calendar_id.clone())),
        }
    }
}
