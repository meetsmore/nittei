use axum::{Extension, Json, extract::Path};
use nittei_api_structs::get_event::*;
use nittei_domain::{Account, CalendarEvent, ID, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Policy, account_can_modify_event},
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    get,
    tag = "Event",
    path = "/api/v1/user/events/{event_id}",
    summary = "Get an event (admin only)",
    params(
        ("event_id" = ID, Path, description = "The id of the event to get"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn get_event_admin_controller(
    Extension(account): Extension<Account>,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;

    let usecase = GetEventUseCase {
        user_id: e.user_id,
        event_id: e.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| Json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

#[utoipa::path(
    get,
    tag = "Event",
    path = "/api/v1/events/{event_id}",
    summary = "Get an event (user only)",
    params(
        ("event_id" = ID, Path, description = "The id of the event to get"),
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn get_event_controller(
    Extension((user, _policy)): Extension<(User, Policy)>,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let usecase = GetEventUseCase {
        event_id: path_params.event_id.clone(),
        user_id: user.id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar_event| Json(APIResponse::new(calendar_event)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetEventUseCase {
    pub event_id: ID,
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
                "The calendar event with id: {}, was not found.",
                event_id
            )),
        }
    }
}

#[async_trait::async_trait]
impl UseCase for GetEventUseCase {
    type Response = CalendarEvent;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEvent";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let e = ctx.repos.events.find(&self.event_id).await.map_err(|e| {
            tracing::error!("[get_event] Error finding event: {:?}", e);
            UseCaseError::InternalError
        })?;
        match e {
            Some(event) if event.user_id == self.user_id => Ok(event),
            _ => Err(UseCaseError::NotFound(self.event_id.clone())),
        }
    }
}
