use axum::{Extension, Json, extract::Path};
use nittei_api_structs::get_calendar::{APIResponse, PathParams};
use nittei_domain::{Account, Calendar, ID, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Policy, account_can_modify_calendar},
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/user/calendar/{calendar_id}",
    summary = "Get a calendar (admin only)",
    security(
        ("api_key" = [])
    ),
    params(
        ("calendar_id" = ID, Path, description = "The id of the calendar to get"),
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn get_calendar_admin_controller(
    Extension(account): Extension<Account>,
    path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = GetCalendarUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| Json(APIResponse::new(calendar)))
        .map_err(NitteiError::from)
}

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/calendar/{calendar_id}",
    summary = "Get a calendar",
    params(
        ("calendar_id" = ID, Path, description = "The id of the calendar to get"),
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn get_calendar_controller(
    Extension((user, _policy)): Extension<(User, Policy)>,
    path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let usecase = GetCalendarUseCase {
        user_id: user.id.clone(),
        calendar_id: path.calendar_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| Json(APIResponse::new(calendar)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct GetCalendarUseCase {
    pub user_id: ID,
    pub calendar_id: ID,
}

#[derive(Debug)]
enum UseCaseError {
    InternalError,
    NotFound(ID),
}
impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::NotFound(calendar_id) => Self::NotFound(format!(
                "The calendar with id: {}, was not found.",
                calendar_id
            )),
        }
    }
}

#[async_trait::async_trait]
impl UseCase for GetCalendarUseCase {
    type Response = Calendar;

    type Error = UseCaseError;

    const NAME: &'static str = "GetCalendar";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let cal = ctx
            .repos
            .calendars
            .find(&self.calendar_id)
            .await
            .map_err(|e| {
                tracing::error!("[get_calendar] Error finding calendar: {:?}", e);
                UseCaseError::InternalError
            })?;
        match cal {
            Some(cal) if cal.user_id == self.user_id => Ok(cal),
            _ => Err(UseCaseError::NotFound(self.calendar_id.clone())),
        }
    }
}
