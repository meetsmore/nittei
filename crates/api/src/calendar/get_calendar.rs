use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::get_calendar::{APIResponse, PathParams};
use nittei_domain::{Calendar, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_calendar, protect_admin_route, protect_route},
        usecase::{UseCase, execute},
    },
};

pub async fn get_calendar_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = GetCalendarUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
        .map_err(NitteiError::from)
}

pub async fn get_calendar_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetCalendarUseCase {
        user_id: user.id.clone(),
        calendar_id: path.calendar_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|calendar| HttpResponse::Ok().json(APIResponse::new(calendar)))
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

#[async_trait::async_trait(?Send)]
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
            .map_err(|_| UseCaseError::InternalError)?;
        match cal {
            Some(cal) if cal.user_id == self.user_id => Ok(cal),
            _ => Err(UseCaseError::NotFound(self.calendar_id.clone())),
        }
    }
}
