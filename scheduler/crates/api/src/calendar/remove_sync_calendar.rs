use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::remove_sync_calendar::{APIResponse, PathParams, RequestBody};
use nettu_scheduler_domain::{IntegrationProvider, ID};
use nettu_scheduler_infra::NettuContext;

use crate::{
    error::NettuError,
    shared::{
        auth::{account_can_modify_user, protect_account_route, Permission},
        usecase::{execute, PermissionBoundary, UseCase},
    },
};

fn error_handler(e: UseCaseError) -> NettuError {
    match e {
        UseCaseError::StorageError => NettuError::InternalError,
        UseCaseError::SyncNotFound => {
            NettuError::NotFound("The given calendar sync was not found.".to_string())
        }
    }
}

pub async fn remove_sync_calendar_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    // Check if user exists and can be modified by the account
    account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let body = body.0;
    let usecase = RemoveSyncCalendarUseCase {
        calendar_id: body.calendar_id,
        ext_calendar_id: body.ext_calendar_id,
        provider: body.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().json(APIResponse::from("Calendar sync created")))
        .map_err(error_handler)
}

#[derive(Debug)]
struct RemoveSyncCalendarUseCase {
    pub provider: IntegrationProvider,
    pub calendar_id: ID,
    pub ext_calendar_id: String,
}

#[derive(Debug)]
enum UseCaseError {
    SyncNotFound,
    StorageError,
}

impl From<anyhow::Error> for UseCaseError {
    fn from(_: anyhow::Error) -> Self {
        UseCaseError::StorageError
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for RemoveSyncCalendarUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "RemoveSyncCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        // Check if calendar sync exists
        let sync_calendar = ctx
            .repos
            .calendar_synced
            .find_by_calendar(&self.calendar_id)
            .await?
            .into_iter()
            .find(|c| c.provider == self.provider && c.ext_calendar_id == self.ext_calendar_id)
            .ok_or(UseCaseError::SyncNotFound)?;

        ctx.repos.calendar_synced.delete(&sync_calendar).await?;
        Ok(())
    }
}

impl PermissionBoundary for RemoveSyncCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendar]
    }
}
