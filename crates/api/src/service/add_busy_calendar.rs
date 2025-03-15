use axum::{
    Extension,
    Json,
    extract::Path,
    http::HeaderMap,
};
use nittei_api_structs::add_busy_calendar::*;
use nittei_domain::{
    Account,
    BusyCalendarProvider,
    ID,
    IntegrationProvider,
    providers::{google::GoogleCalendarAccessRole, outlook::OutlookCalendarAccessRole},
};
use nittei_infra::{
    BusyCalendarIdentifier,
    ExternalBusyCalendarIdentifier,
    NitteiContext,
    google_calendar::GoogleCalendarProvider,
    outlook_calendar::OutlookCalendarProvider,
};

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn add_busy_calendar_controller(
    headers: HeaderMap,
    mut path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
    body: Json<RequestBody>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

    let body = body.0;
    let usecase = AddBusyCalendarUseCase {
        account,
        service_id: std::mem::take(&mut path.service_id),
        user_id: std::mem::take(&mut path.user_id),
        busy: body.busy.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|_| Json(APIResponse::from("Busy calendar added to service user")))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct AddBusyCalendarUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
    pub busy: BusyCalendarProvider,
}

#[derive(Debug)]
enum UseCaseError {
    StorageError,
    UserNotFound,
    CalendarAlreadyRegistered,
    CalendarNotFound,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::CalendarNotFound => {
                Self::NotFound("The requested calendar was not found or user is missing permissions to read the calendar".into())
            }
            UseCaseError::UserNotFound => {
                Self::NotFound("The specified user was not found".into())
            }
            UseCaseError::CalendarAlreadyRegistered => Self::Conflict(
                "The busy calendar is already registered on the service user".into(),
            ),
        }
    }
}

#[async_trait::async_trait]
impl UseCase for AddBusyCalendarUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "AddBusyCalendar";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let user = ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?
            .ok_or(UseCaseError::UserNotFound)?;

        // Check if busy calendar already exists
        match &self.busy {
            BusyCalendarProvider::Google(g_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: g_cal_id.clone(),
                    provider: IntegrationProvider::Google,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                if ctx
                    .repos
                    .service_user_busy_calendars
                    .exists_ext(identifier)
                    .await
                    .unwrap_or(false)
                {
                    return Err(UseCaseError::CalendarAlreadyRegistered);
                }
            }
            BusyCalendarProvider::Outlook(o_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: o_cal_id.clone(),
                    provider: IntegrationProvider::Outlook,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                if ctx
                    .repos
                    .service_user_busy_calendars
                    .exists_ext(identifier)
                    .await
                    .unwrap_or(false)
                {
                    return Err(UseCaseError::CalendarAlreadyRegistered);
                }
            }
            BusyCalendarProvider::Nittei(n_cal_id) => {
                let identifier = BusyCalendarIdentifier {
                    calendar_id: n_cal_id.clone(),
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                if ctx
                    .repos
                    .service_user_busy_calendars
                    .exists(identifier)
                    .await
                    .unwrap_or(false)
                {
                    return Err(UseCaseError::CalendarAlreadyRegistered);
                }
            }
        }

        // Validate calendar permissions
        match &self.busy {
            BusyCalendarProvider::Google(g_cal_id) => {
                let provider = GoogleCalendarProvider::new(&user, ctx)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                let g_calendars = provider
                    .list(GoogleCalendarAccessRole::FreeBusyReader)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                if !g_calendars
                    .items
                    .into_iter()
                    .any(|g_calendar| g_calendar.id == *g_cal_id)
                {
                    return Err(UseCaseError::CalendarNotFound);
                }
            }
            BusyCalendarProvider::Outlook(o_cal_id) => {
                let provider = OutlookCalendarProvider::new(&user, ctx)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                let o_calendars = provider
                    .list(OutlookCalendarAccessRole::Reader)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                if !o_calendars
                    .into_iter()
                    .any(|o_calendar| o_calendar.id == *o_cal_id)
                {
                    return Err(UseCaseError::CalendarNotFound);
                }
            }
            BusyCalendarProvider::Nittei(n_cal_id) => {
                match ctx.repos.calendars.find(n_cal_id).await {
                    Ok(Some(cal)) if cal.user_id == user.id => (),
                    Ok(_) => return Err(UseCaseError::CalendarNotFound),
                    Err(_) => return Err(UseCaseError::StorageError),
                }
            }
        }

        // Insert busy calendar
        match &self.busy {
            BusyCalendarProvider::Google(g_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: g_cal_id.clone(),
                    provider: IntegrationProvider::Google,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                ctx.repos
                    .service_user_busy_calendars
                    .insert_ext(identifier)
                    .await
                    .map_err(|_| UseCaseError::StorageError)
            }
            BusyCalendarProvider::Outlook(o_cal_id) => {
                let identifier = ExternalBusyCalendarIdentifier {
                    ext_calendar_id: o_cal_id.clone(),
                    provider: IntegrationProvider::Outlook,
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                ctx.repos
                    .service_user_busy_calendars
                    .insert_ext(identifier)
                    .await
                    .map_err(|_| UseCaseError::StorageError)
            }
            BusyCalendarProvider::Nittei(n_cal_id) => {
                let identifier = BusyCalendarIdentifier {
                    calendar_id: n_cal_id.clone(),
                    service_id: self.service_id.clone(),
                    user_id: user.id.clone(),
                };
                ctx.repos
                    .service_user_busy_calendars
                    .insert(identifier)
                    .await
                    .map_err(|_| UseCaseError::StorageError)
            }
        }
    }
}
