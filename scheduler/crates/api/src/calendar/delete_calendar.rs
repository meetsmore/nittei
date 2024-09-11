use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::delete_calendar::{APIResponse, PathParams};
use nittei_domain::{Calendar, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_calendar, protect_account_route, protect_route, Permission},
        usecase::{execute, execute_with_policy, PermissionBoundary, UseCase},
    },
};

pub async fn delete_calendar_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = DeleteCalendarUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(NitteiError::from)
}

pub async fn delete_calendar_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteCalendarUseCase {
        user_id: user.id,
        calendar_id: path.calendar_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
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

#[async_trait::async_trait(?Send)]
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
