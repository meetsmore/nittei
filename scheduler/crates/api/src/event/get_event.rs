use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_event::*;
use nettu_scheduler_domain::{CalendarEvent, ID};
use nettu_scheduler_infra::NettuContext;

use crate::{
    error::NettuError,
    shared::{
        auth::{account_can_modify_event, protect_account_route, protect_route},
        usecase::{execute, UseCase},
    },
};

pub async fn get_event_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;

    let usecase = GetEventUseCase {
        user_id: e.user_id,
        event_id: e.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(NettuError::from)
}

pub async fn get_event_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetEventUseCase {
        event_id: path_params.event_id.clone(),
        user_id: user.id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar_event| HttpResponse::Ok().json(APIResponse::new(calendar_event)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
pub struct GetEventUseCase {
    pub event_id: ID,
    pub user_id: ID,
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(ID),
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(event_id) => Self::NotFound(format!(
                "The calendar event with id: {}, was not found.",
                event_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventUseCase {
    type Response = CalendarEvent;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEvent";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let e = ctx.repos.events.find(&self.event_id).await;
        match e {
            Some(event) if event.user_id == self.user_id => Ok(event),
            _ => Err(UseCaseError::NotFound(self.event_id.clone())),
        }
    }
}
