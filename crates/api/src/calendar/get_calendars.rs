use axum::{
    Extension,
    Json,
    extract::{Path, Query},
    http::HeaderMap,
};
use nittei_api_structs::get_calendars_by_user::{
    GetCalendarsByUserAPIResponse,
    PathParams,
    QueryParams,
};
use nittei_domain::{Calendar, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_route,
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/calendar/{user_id}",
    summary = "Get calendars for a user (admin only)",
    params(
        ("user_id" = ID, Path, description = "The id of the user to get calendars for"),
        ("key" = Option<String>, Query, description = "Optional key of the calendar to get"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = GetCalendarsByUserAPIResponse)
    )
)]
pub async fn get_calendars_admin_controller(
    query: Query<QueryParams>,
    path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<GetCalendarsByUserAPIResponse>, NitteiError> {
    let usecase = GetCalendarsUseCase {
        user_id: path.user_id.clone(),
        key: query.key.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| Json(GetCalendarsByUserAPIResponse::new(calendars)))
        .map_err(NitteiError::from)
}

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/calendar",
    summary = "Get calendars for a user",
    params(
        ("user_id" = ID, Path, description = "The id of the user to get calendars for"),
        ("key" = Option<String>, Query, description = "Optional key of the calendar to get"),
    ),
    responses(
        (status = 200, body = GetCalendarsByUserAPIResponse)
    )
)]
/// Get calendars for a user
pub async fn get_calendars_controller(
    headers: HeaderMap,
    query: Query<QueryParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<GetCalendarsByUserAPIResponse>, NitteiError> {
    let (user, _policy) = protect_route(&headers, &ctx).await?;

    let usecase = GetCalendarsUseCase {
        user_id: user.id.clone(),
        key: query.key.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| Json(GetCalendarsByUserAPIResponse::new(calendars)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct GetCalendarsUseCase {
    pub user_id: ID,
    pub key: Option<String>,
}

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
impl UseCase for GetCalendarsUseCase {
    type Response = Vec<Calendar>;

    type Error = UseCaseError;

    const NAME: &'static str = "GetCalendar";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        match &self.key {
            Some(key) => ctx
                .repos
                .calendars
                .find_by_user_and_key(&self.user_id, key)
                .await
                .map_err(|_| UseCaseError::InternalError)
                .map(|calendar| calendar.into_iter().collect()),
            None => ctx
                .repos
                .calendars
                .find_by_user(&self.user_id)
                .await
                .map_err(|_| UseCaseError::InternalError),
        }
    }
}
