use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::remove_busy_calendar::*;
use nettu_scheduler_domain::{Account, BusyCalendar, IntegrationProvider, ID};
use nettu_scheduler_infra::{BusyCalendarIdentifier, ExternalBusyCalendarIdentifier, NettuContext};

use crate::{
    error::NettuError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn remove_busy_calendar_controller(
    http_req: HttpRequest,
    mut path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let body = body.0;

    let usecase = RemoveBusyCalendarUseCase {
        account,
        service_id: std::mem::take(&mut path.service_id),
        user_id: std::mem::take(&mut path.user_id),
        busy: body.busy,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().json(APIResponse::from("Busy calendar added to service user")))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct RemoveBusyCalendarUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
    pub busy: BusyCalendar,
}

#[derive(Debug)]
enum UseCaseError {
    StorageError,
    UserNotFound,
    BusyCalendarNotFound,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::UserNotFound => Self::NotFound("The specified user was not found".into()),
            UseCaseError::BusyCalendarNotFound => {
                Self::NotFound("The busy calendar is not registered on the service user".into())
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for RemoveBusyCalendarUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "RemoveBusyCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let user = ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?
            .ok_or(UseCaseError::UserNotFound)?;

        // Check if busy calendar exists
        match &self.busy {
            BusyCalendar::Google(g_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: g_cal_id.clone(),
                    provider: IntegrationProvider::Google,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                if !ctx
                    .repos
                    .service_user_busy_calendars
                    .exists_ext(identifier)
                    .await
                    .unwrap_or(false)
                {
                    return Err(UseCaseError::BusyCalendarNotFound);
                }
            }
            BusyCalendar::Outlook(o_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: o_cal_id.clone(),
                    provider: IntegrationProvider::Outlook,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                if !ctx
                    .repos
                    .service_user_busy_calendars
                    .exists_ext(identifier)
                    .await
                    .unwrap_or(false)
                {
                    return Err(UseCaseError::BusyCalendarNotFound);
                }
            }
            BusyCalendar::Nettu(n_cal_id) => {
                let identifier = BusyCalendarIdentifier {
                    calendar_id: n_cal_id.clone(),
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                if !ctx
                    .repos
                    .service_user_busy_calendars
                    .exists(identifier)
                    .await
                    .unwrap_or(false)
                {
                    return Err(UseCaseError::BusyCalendarNotFound);
                }
            }
        }

        // Delete busy calendar
        match &self.busy {
            BusyCalendar::Google(g_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: g_cal_id.clone(),
                    provider: IntegrationProvider::Google,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                ctx.repos
                    .service_user_busy_calendars
                    .delete_ext(identifier)
                    .await
                    .map_err(|_| UseCaseError::StorageError)
            }
            BusyCalendar::Outlook(o_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: o_cal_id.clone(),
                    provider: IntegrationProvider::Outlook,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                ctx.repos
                    .service_user_busy_calendars
                    .delete_ext(identifier)
                    .await
                    .map_err(|_| UseCaseError::StorageError)
            }
            BusyCalendar::Nettu(n_cal_id) => {
                let identifier = BusyCalendarIdentifier {
                    calendar_id: n_cal_id.clone(),
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                ctx.repos
                    .service_user_busy_calendars
                    .delete(identifier)
                    .await
                    .map_err(|_| UseCaseError::StorageError)
            }
        }
    }
}
