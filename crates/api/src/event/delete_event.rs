use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::delete_event::*;
use nittei_domain::{CalendarEvent, ID, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{
            Permission,
            account_can_modify_event,
            account_can_modify_user,
            protect_admin_route,
            protect_route,
        },
        usecase::{PermissionBoundary, UseCase, execute, execute_with_policy},
    },
};

pub async fn delete_event_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;
    let user = account_can_modify_user(&account, &e.user_id, &ctx).await?;

    let usecase = DeleteEventUseCase {
        user,
        event_id: e.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

pub async fn delete_event_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteEventUseCase {
        user,
        event_id: path_params.event_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct DeleteEventUseCase {
    pub user: User,
    pub event_id: ID,
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
            UseCaseError::NotFound(event_id) => Self::NotFound(format!(
                "The calendar event with id: {}, was not found.",
                event_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteEventUseCase {
    type Response = CalendarEvent;

    type Error = UseCaseError;

    const NAME: &'static str = "DeleteEvent";

    // TODO: use only one db call
    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let event = ctx
            .repos
            .events
            .find(&self.event_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        let e = match event {
            Some(e) if e.user_id == self.user.id => e,
            _ => return Err(UseCaseError::NotFound(self.event_id.clone())),
        };

        ctx.repos
            .events
            .delete(&e.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;

        Ok(e)
    }
}

impl PermissionBoundary for DeleteEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteCalendarEvent]
    }
}
