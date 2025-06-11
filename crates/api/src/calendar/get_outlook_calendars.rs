use axum::{
    Extension,
    Json,
    extract::{Path, Query},
    http::HeaderMap,
};
use nittei_api_structs::get_outlook_calendars::{
    GetOutlookCalendarsAPIResponse,
    PathParams,
    QueryParams,
};
use nittei_domain::{
    Account,
    ID,
    User,
    providers::outlook::{OutlookCalendar, OutlookCalendarAccessRole},
};
use nittei_infra::{NitteiContext, outlook_calendar::OutlookCalendarProvider};

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_user, protect_route},
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/user/{user_id}/calendar/provider/outlook",
    summary = "Get outlook calendars for a user (admin only)",
    params(
        ("user_id" = ID, Path, description = "The id of the user to get outlook calendars for"),
        ("min_access_role" = OutlookCalendarAccessRole, Query, description = "The minimum access role to get outlook calendars for"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = GetOutlookCalendarsAPIResponse)
    )
)]
pub async fn get_outlook_calendars_admin_controller(
    Extension(account): Extension<Account>,
    path: Path<PathParams>,
    query: Query<QueryParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<GetOutlookCalendarsAPIResponse>, NitteiError> {
    let user = account_can_modify_user(&account, &path.user_id, &ctx).await?;

    let usecase = GetOutlookCalendarsUseCase {
        user,
        min_access_role: query.0.min_access_role,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| Json(GetOutlookCalendarsAPIResponse::new(calendars)))
        .map_err(NitteiError::from)
}

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/calendar/provider/outlook",
    summary = "Get outlook calendars for a user",
    params(
        ("min_access_role" = OutlookCalendarAccessRole, Query, description = "The minimum access role to get outlook calendars for"),
    ),
    responses(
        (status = 200, body = GetOutlookCalendarsAPIResponse)
    )
)]
pub async fn get_outlook_calendars_controller(
    headers: HeaderMap,
    query: Query<QueryParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<GetOutlookCalendarsAPIResponse>, NitteiError> {
    let (user, _policy) = protect_route(&headers, &ctx).await?;

    let usecase = GetOutlookCalendarsUseCase {
        user,
        min_access_role: query.0.min_access_role,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| Json(GetOutlookCalendarsAPIResponse::new(calendars)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct GetOutlookCalendarsUseCase {
    pub user: User,
    pub min_access_role: OutlookCalendarAccessRole,
}

#[derive(Debug)]
enum UseCaseError {
    UserNotConnectedToOutlook,
    OutlookQuery,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::UserNotConnectedToOutlook => {
                Self::BadClientData("The user is not connected to outlook.".into())
            }
            UseCaseError::OutlookQuery => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for GetOutlookCalendarsUseCase {
    type Response = Vec<OutlookCalendar>;

    type Error = UseCaseError;

    const NAME: &'static str = "GetOutlookCalendars";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let provider = OutlookCalendarProvider::new(&self.user, ctx)
            .await
            .map_err(|_| UseCaseError::UserNotConnectedToOutlook)?;

        provider
            .list(self.min_access_role.clone())
            .await
            .map_err(|_| UseCaseError::OutlookQuery)
    }
}
