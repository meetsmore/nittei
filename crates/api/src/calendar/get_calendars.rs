use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::get_calendars_by_user::{APIResponse, PathParams};
use nittei_domain::{Calendar, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{protect_account_route, protect_route},
        usecase::{execute, UseCase},
    },
};

pub async fn get_calendars_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let _account = protect_account_route(&http_req, &ctx).await?;

    let usecase = GetCalendarsUseCase {
        user_id: path.user_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| HttpResponse::Ok().json(APIResponse::new(calendars)))
        .map_err(NitteiError::from)
}

pub async fn get_calendars_controller(
    http_req: HttpRequest,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetCalendarsUseCase {
        user_id: user.id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| HttpResponse::Ok().json(APIResponse::new(calendars)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct GetCalendarsUseCase {
    pub user_id: ID,
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
        ctx.repos
            .calendars
            .find_by_user(&self.user_id)
            .await
            .map_err(|_| UseCaseError::InternalError)
    }
}
