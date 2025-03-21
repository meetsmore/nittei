use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::get_google_calendars::{APIResponse, PathParams, QueryParams};
use nittei_domain::{
    User,
    providers::google::{GoogleCalendarAccessRole, GoogleCalendarListEntry},
};
use nittei_infra::{NitteiContext, google_calendar::GoogleCalendarProvider};

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_user, protect_admin_route, protect_route},
        usecase::{UseCase, execute},
    },
};

pub async fn get_google_calendars_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    query: web::Query<QueryParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path.user_id, &ctx).await?;

    let usecase = GetGoogleCalendarsUseCase {
        user,
        min_access_role: query.0.min_access_role,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| HttpResponse::Ok().json(APIResponse::new(calendars)))
        .map_err(NitteiError::from)
}

pub async fn get_google_calendars_controller(
    http_req: HttpRequest,
    query: web::Query<QueryParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetGoogleCalendarsUseCase {
        user,
        min_access_role: query.0.min_access_role,
    };

    execute(usecase, &ctx)
        .await
        .map(|calendars| HttpResponse::Ok().json(APIResponse::new(calendars)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct GetGoogleCalendarsUseCase {
    pub user: User,
    pub min_access_role: GoogleCalendarAccessRole,
}

#[derive(Debug)]
enum UseCaseError {
    UserNotConnectedToGoogle,
    GoogleQuery,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::UserNotConnectedToGoogle => {
                Self::BadClientData("The user is not connected to google.".into())
            }
            UseCaseError::GoogleQuery => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for GetGoogleCalendarsUseCase {
    type Response = Vec<GoogleCalendarListEntry>;

    type Error = UseCaseError;

    const NAME: &'static str = "GetGoogleCalendars";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let provider = GoogleCalendarProvider::new(&self.user, ctx)
            .await
            .map_err(|_| UseCaseError::UserNotConnectedToGoogle)?;

        provider
            .list(self.min_access_role.clone())
            .await
            .map_err(|_| UseCaseError::GoogleQuery)
            .map(|res| res.items)
    }
}
