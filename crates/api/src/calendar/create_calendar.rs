use axum::{Extension, Json, http::StatusCode};
use axum_valid::Valid;
use chrono::Weekday;
use chrono_tz::Tz;
use nittei_api_structs::create_calendar::{APIResponse, CreateCalendarRequestBody};
use nittei_domain::{Calendar, CalendarSettings, ID, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Permission, Policy},
        usecase::{PermissionBoundary, UseCase, execute, execute_with_policy},
    },
};

#[utoipa::path(
    post,
    tag = "Calendar",
    path = "/api/v1/user/{user_id}/calendar",
    summary = "Create a calendar (admin only)",
    security(
        ("api_key" = [])
    ),
    request_body(
        content = CreateCalendarRequestBody,
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn create_calendar_admin_controller(
    Extension(user): Extension<User>,
    Extension(ctx): Extension<NitteiContext>,
    mut body: Valid<Json<CreateCalendarRequestBody>>,
) -> Result<(StatusCode, Json<APIResponse>), NitteiError> {
    let usecase = CreateCalendarUseCase {
        user_id: user.id,
        account_id: user.account_id,
        week_start: body.0.week_start,
        name: body.0.name.take(),
        key: body.0.key.take(),
        timezone: body.0.timezone,
        metadata: body.0.metadata.take(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| (StatusCode::CREATED, Json(APIResponse::new(calendar))))
        .map_err(NitteiError::from)
}

#[utoipa::path(
    post,
    tag = "Calendar",
    path = "/api/v1/calendar",
    summary = "Create a calendar",
    request_body(
        content = CreateCalendarRequestBody,
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn create_calendar_controller(
    Extension((user, policy)): Extension<(User, Policy)>,
    Extension(ctx): Extension<NitteiContext>,
    mut body: Valid<Json<CreateCalendarRequestBody>>,
) -> Result<(StatusCode, Json<APIResponse>), NitteiError> {
    let usecase = CreateCalendarUseCase {
        user_id: user.id,
        account_id: user.account_id,
        week_start: body.0.week_start,
        name: body.0.name.take(),
        key: body.0.key.take(),
        timezone: body.0.timezone,
        metadata: body.0.metadata.take(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| (StatusCode::CREATED, Json(APIResponse::new(calendar))))
        .map_err(NitteiError::from)
}

/// Use case for creating a calendar
#[derive(Debug)]
struct CreateCalendarUseCase {
    pub user_id: ID,
    pub account_id: ID,
    pub week_start: Weekday,
    pub name: Option<String>,
    pub key: Option<String>,
    pub timezone: Tz,
    pub metadata: Option<serde_json::Value>,
}

/// Errors for the create calendar use case
#[derive(Debug)]
enum UseCaseError {
    InternalError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for CreateCalendarUseCase {
    type Response = Calendar;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateCalendar";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let settings = CalendarSettings {
            week_start: self.week_start,
            timezone: self.timezone,
        };
        let mut calendar = Calendar::new(
            &self.user_id,
            &self.account_id,
            self.name.clone(),
            self.key.clone(),
        );
        calendar.settings = settings;
        calendar.metadata = self.metadata.clone();

        ctx.repos
            .calendars
            .insert(&calendar)
            .await
            .map(|_| calendar)
            .map_err(|e| {
                tracing::error!("[create_calendar] Error inserting calendar: {:?}", e);
                UseCaseError::InternalError
            })
    }
}

impl PermissionBoundary for CreateCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateCalendar]
    }
}
