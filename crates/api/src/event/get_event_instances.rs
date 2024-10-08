use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::get_event_instances::*;
use nittei_domain::{CalendarEvent, EventInstance, TimeSpan, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_event, protect_account_route, protect_route},
        usecase::{execute, UseCase},
    },
};

pub async fn get_event_instances_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    query_params: web::Query<QueryParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;

    let usecase = GetEventInstancesUseCase {
        user_id: e.user_id,
        event_id: e.id,
        timespan: query_params.0,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.event, usecase_res.instances))
        })
        .map_err(NitteiError::from)
}

pub async fn get_event_instances_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    query_params: web::Query<QueryParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetEventInstancesUseCase {
        user_id: user.id.clone(),
        event_id: path_params.event_id.clone(),
        timespan: query_params.0,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.event, usecase_res.instances))
        })
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetEventInstancesUseCase {
    pub user_id: ID,
    pub event_id: ID,
    pub timespan: QueryParams,
}

#[derive(Debug)]
pub enum UseCaseError {
    InternalError,
    NotFound(String, ID),
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

#[derive(Debug)]
pub struct UseCaseResponse {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventInstancesUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEventInstances";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let e = ctx
            .repos
            .events
            .find(&self.event_id)
            .await
            .map_err(|_| UseCaseError::InternalError)?;
        match e {
            Some(event) if self.user_id == event.user_id => {
                let calendar = match ctx.repos.calendars.find(&event.calendar_id).await {
                    Ok(Some(cal)) => cal,
                    Ok(None) => {
                        return Err(UseCaseError::NotFound("Calendar".into(), event.calendar_id))
                    }
                    Err(_) => {
                        return Err(UseCaseError::InternalError);
                    }
                };

                let timespan = TimeSpan::new(self.timespan.start_time, self.timespan.end_time);
                if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
                    return Err(UseCaseError::InvalidTimespan);
                }

                let instances = event.expand(Some(&timespan), &calendar.settings);
                Ok(UseCaseResponse { event, instances })
            }
            _ => Err(UseCaseError::NotFound(
                "Calendar Event".into(),
                self.event_id.clone(),
            )),
        }
    }
}
