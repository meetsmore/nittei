use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Weekday;
use chrono_tz::Tz;
use nittei_api_structs::create_calendar::{APIResponse, PathParams, RequestBody};
use nittei_domain::{Calendar, CalendarSettings, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_user, protect_account_route, protect_route, Permission},
        usecase::{execute, execute_with_policy, PermissionBoundary, UseCase},
    },
};

pub async fn create_calendar_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body: actix_web_validator::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let usecase = CreateCalendarUseCase {
        user_id: user.id,
        account_id: account.id,
        week_start: body.0.week_start,
        name: body.0.name.clone(),
        key: body.0.key.clone(),
        timezone: body.0.timezone,
        metadata: body.0.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Created().json(APIResponse::new(calendar)))
        .map_err(NitteiError::from)
}

pub async fn create_calendar_controller(
    http_req: HttpRequest,
    body: actix_web_validator::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateCalendarUseCase {
        user_id: user.id,
        account_id: user.account_id,
        week_start: body.0.week_start,
        name: body.0.name.clone(),
        key: body.0.key.clone(),
        timezone: body.0.timezone,
        metadata: body.0.metadata,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| HttpResponse::Created().json(APIResponse::new(calendar)))
        .map_err(NitteiError::from)
}

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

#[derive(Debug)]
enum UseCaseError {
    InternalError,
    UserNotFound,
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::UserNotFound => {
                Self::NotFound("The requested user was not found.".to_string())
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateCalendarUseCase {
    type Response = Calendar;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateCalendar";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let user = ctx
            .repos
            .users
            .find(&self.user_id)
            .await
            .map_err(|_| UseCaseError::InternalError)?;

        let user = match user {
            Some(user) if user.account_id == self.account_id => user,
            _ => return Err(UseCaseError::UserNotFound),
        };

        let settings = CalendarSettings {
            week_start: self.week_start,
            timezone: self.timezone,
        };
        let mut calendar = Calendar::new(
            &self.user_id,
            &user.account_id,
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
            .map_err(|_| UseCaseError::StorageError)
    }
}

impl PermissionBoundary for CreateCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateCalendar]
    }
}
