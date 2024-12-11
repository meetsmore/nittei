use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::get_event_group::*;
use nittei_domain::{event_group::EventGroup, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_event_group, protect_account_route},
        usecase::{execute, UseCase},
    },
};

pub async fn get_event_group_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event_group(&account, &path_params.event_group_id, &ctx).await?;

    let usecase = GetEventGroupUseCase {
        user_id: e.user_id,
        event_group_id: e.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|event_group| HttpResponse::Ok().json(APIResponse::new(event_group)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetEventGroupUseCase {
    pub event_group_id: ID,
    pub user_id: ID,
}

#[derive(Debug)]
pub enum UseCaseError {
    InternalError,
    NotFound(ID),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::NotFound(event_id) => Self::NotFound(format!(
                "The event group with id: {}, was not found.",
                event_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventGroupUseCase {
    type Response = EventGroup;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEvent";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let e = ctx
            .repos
            .event_groups
            .find(&self.event_group_id)
            .await
            .map_err(|_| UseCaseError::InternalError)?;
        match e {
            Some(event_group) if event_group.user_id == self.user_id => Ok(event_group),
            _ => Err(UseCaseError::NotFound(self.event_group_id.clone())),
        }
    }
}
