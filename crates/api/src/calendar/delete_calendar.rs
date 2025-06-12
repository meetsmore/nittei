use axum::{Extension, Json, extract::Path};
use nittei_api_structs::delete_calendar::{APIResponse, PathParams};
use nittei_domain::{Account, Calendar, ID, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Permission, Policy, account_can_modify_calendar},
        usecase::{PermissionBoundary, UseCase, execute, execute_with_policy},
    },
};

#[utoipa::path(
    delete,
    tag = "Calendar",
    path = "/api/v1/user/calendar/{calendar_id}",
    summary = "Delete a calendar (admin only)",
    params(
        ("calendar_id" = ID, Path, description = "The id of the calendar to delete"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn delete_calendar_admin_controller(
    Extension(account): Extension<Account>,
    path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = DeleteCalendarUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| Json(APIResponse::new(calendar)))
        .map_err(NitteiError::from)
}

#[utoipa::path(
    delete,
    tag = "Calendar",
    path = "/api/v1/calendar/{calendar_id}",
    summary = "Delete a calendar",
    params(
        ("calendar_id" = ID, Path, description = "The id of the calendar to delete"),
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn delete_calendar_controller(
    Extension((user, policy)): Extension<(User, Policy)>,
    path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let usecase = DeleteCalendarUseCase {
        user_id: user.id,
        calendar_id: path.calendar_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| Json(APIResponse::new(calendar)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub enum UseCaseError {
    InternalError,
    NotFound(ID),
    UnableToDelete,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::NotFound(calendar_id) => Self::NotFound(format!(
                "The calendar with id: {}, was not found.",
                calendar_id
            )),
            UseCaseError::UnableToDelete => Self::InternalError,
        }
    }
}

#[derive(Debug)]
pub struct DeleteCalendarUseCase {
    calendar_id: ID,
    user_id: ID,
}

#[async_trait::async_trait]
impl UseCase for DeleteCalendarUseCase {
    type Response = Calendar;

    type Error = UseCaseError;

    const NAME: &'static str = "DeleteCalendar";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let calendar = ctx
            .repos
            .calendars
            .find(&self.calendar_id)
            .await
            .map_err(|_| UseCaseError::InternalError)?;
        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => ctx
                .repos
                .calendars
                .delete(&calendar.id)
                .await
                .map(|_| calendar)
                .map_err(|_| UseCaseError::UnableToDelete),
            _ => Err(UseCaseError::NotFound(self.calendar_id.clone())),
        }
    }
}

impl PermissionBoundary for DeleteCalendarUseCase {
    fn permissions(&self) -> Vec<crate::shared::auth::Permission> {
        vec![Permission::DeleteCalendar]
    }
}
