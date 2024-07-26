use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::add_sync_calendar::{APIResponse, PathParams, RequestBody};
use nettu_scheduler_domain::{
    providers::{google::GoogleCalendarAccessRole, outlook::OutlookCalendarAccessRole},
    IntegrationProvider,
    SyncedCalendar,
    User,
    ID,
};
use nettu_scheduler_infra::{
    google_calendar::GoogleCalendarProvider,
    outlook_calendar::OutlookCalendarProvider,
    NettuContext,
};

use crate::{
    error::NettuError,
    shared::{
        auth::{account_can_modify_user, protect_account_route, Permission},
        usecase::{execute, PermissionBoundary, UseCase},
    },
};

pub async fn add_sync_calendar_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let body = body.0;
    let usecase = AddSyncCalendarUseCase {
        user,
        calendar_id: body.calendar_id,
        ext_calendar_id: body.ext_calendar_id,
        provider: body.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().json(APIResponse::from("Calendar sync created")))
        .map_err(NettuError::from)
}

// pub async fn add_sync_calendar_controller(
//     http_req: web::HttpRequest,
//     body: web::Json<RequestBody>,
//     ctx: web::Data<NettuContext>,
// ) -> Result<HttpResponse, NettuError> {
//     let (user, policy) = protect_route(&http_req, &ctx).await?;

//     let body = body.0;

//     let usecase = AddSyncCalendarUseCase {
//         user,
//         calendar_id: body.calendar_id,
//         ext_calendar_id: body.ext_calendar_id,
//         provider: body.provider,
//     };

//     execute_with_policy(usecase, &policy, &ctx)
//         .await
//         .map(|_| HttpResponse::Ok().json(APIResponse::from("Calendar sync created")))
//         .map_err(|e| match e {
//             UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
//             UseCaseErrorContainer::UseCase(e) => error_handler(e),
//         })
// }

#[derive(Debug)]
struct AddSyncCalendarUseCase {
    pub user: User,
    pub provider: IntegrationProvider,
    pub calendar_id: ID,
    pub ext_calendar_id: String,
}

#[derive(Debug)]
enum UseCaseError {
    NoProviderIntegration,
    ExternalCalendarNotFound,
    CalendarAlreadySynced,
    StorageError,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::ExternalCalendarNotFound => Self::NotFound("The external calendar was not found. Make sure it exists and that user has write access to that calendar".into()),
            UseCaseError::CalendarAlreadySynced => Self::Conflict("The calendar is already synced to the given external calendar".into()),
            UseCaseError::NoProviderIntegration => Self::NotFound("The user has not integrated with the given provider".into()),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddSyncCalendarUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "AddSyncCalendar";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        // Check that user has integrated to that provider
        ctx.repos
            .user_integrations
            .find(&self.user.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?
            .into_iter()
            .find(|i| i.provider == self.provider)
            .ok_or(UseCaseError::NoProviderIntegration)?;

        // Check if calendar sync already exists
        if ctx
            .repos
            .calendar_synced
            .find_by_calendar(&self.calendar_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?
            .into_iter()
            .any(|c| c.provider == self.provider && c.ext_calendar_id == self.ext_calendar_id)
        {
            return Err(UseCaseError::CalendarAlreadySynced);
        }

        // Check that user has write access to the given external calendar.
        match self.provider {
            IntegrationProvider::Google => {
                let google_provider = GoogleCalendarProvider::new(&self.user, ctx)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;
                let google_calendars = google_provider
                    .list(GoogleCalendarAccessRole::Writer)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                if !google_calendars
                    .items
                    .into_iter()
                    .map(|c| c.id)
                    .any(|google_calendar_id| google_calendar_id == self.ext_calendar_id)
                {
                    return Err(UseCaseError::ExternalCalendarNotFound);
                }
            }
            IntegrationProvider::Outlook => {
                let outlook_provider = OutlookCalendarProvider::new(&self.user, ctx)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;
                let outlook_calendars = outlook_provider
                    .list(OutlookCalendarAccessRole::Writer)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                if !outlook_calendars
                    .into_iter()
                    .map(|c| c.id)
                    .any(|outlook_calendar_id| outlook_calendar_id == self.ext_calendar_id)
                {
                    return Err(UseCaseError::ExternalCalendarNotFound);
                }
            }
        }

        let synced_calendar = SyncedCalendar {
            calendar_id: self.calendar_id.clone(),
            ext_calendar_id: self.ext_calendar_id.clone(),
            provider: self.provider.clone(),
            user_id: self.user.id.clone(),
        };

        ctx.repos
            .calendar_synced
            .insert(&synced_calendar)
            .await
            .map_err(|_| UseCaseError::StorageError)
    }
}

impl PermissionBoundary for AddSyncCalendarUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendar]
    }
}
