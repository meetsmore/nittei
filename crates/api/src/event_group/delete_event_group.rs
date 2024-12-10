use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::delete_event_group::*;
use nittei_domain::{event_group::EventGroup, User, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_event, account_can_modify_user, protect_account_route},
        usecase::{execute, UseCase},
    },
};

pub async fn delete_event_group_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_group_id, &ctx).await?;
    let user = account_can_modify_user(&account, &e.user_id, &ctx).await?;

    let usecase = DeleteEventGroupUseCase {
        user,
        event_group_id: e.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct DeleteEventGroupUseCase {
    pub user: User,
    pub event_group_id: ID,
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(ID),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::NotFound(event_group_id) => Self::NotFound(format!(
                "The event group with id: {}, was not found.",
                event_group_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteEventGroupUseCase {
    type Response = EventGroup;

    type Error = UseCaseError;

    const NAME: &'static str = "DeleteEvent";

    // TODO: use only one db call
    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let event_group = ctx
            .repos
            .event_groups
            .find(&self.event_group_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        let g = match event_group {
            Some(g) if g.user_id == self.user.id => g,
            _ => return Err(UseCaseError::NotFound(self.event_group_id.clone())),
        };

        ctx.repos
            .event_groups
            .delete(&g.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;

        Ok(g)
    }
}
