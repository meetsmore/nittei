use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use nittei_api_structs::get_calendars_by_user::{APIResponse, PathParams, QueryParams};
use nittei_domain::{Calendar, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{protect_account_route, protect_route},
        usecase::{execute, UseCase},
    },
};

/// Get calendars for a user (admin)
pub async fn get_calendars_admin_controller(
    headers: HeaderMap,
    query: Query<QueryParams>,
    path: Path<PathParams>,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let _account = protect_account_route(&headers, &ctx).await?;

    let usecase = GetCalendarsUseCase {
        user_id: path.user_id.clone(),
        key: query.key.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| Json(APIResponse::new(calendars)))
        .map_err(NitteiError::from)
}

// Get calendars for a user
pub async fn get_calendars_controller(
    headers: HeaderMap,
    query: Query<QueryParams>,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let (user, _policy) = protect_route(&headers, &ctx).await?;

    let usecase = GetCalendarsUseCase {
        user_id: user.id.clone(),
        key: query.key.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| Json(APIResponse::new(calendars)))
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

#[async_trait::async_trait(?Send)]
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
