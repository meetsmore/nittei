use axum::{Extension, Json, extract::Path};
use nittei_api_structs::delete_event::*;
use nittei_domain::{Account, CalendarEvent, ID, IntegrationProvider, User};
use nittei_infra::{
    NitteiContext,
    google_calendar::GoogleCalendarProvider,
    outlook_calendar::OutlookCalendarProvider,
};
use nittei_utils::config::APP_CONFIG;
use tracing::error;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Permission, Policy, account_can_modify_event, account_can_modify_user},
        usecase::{PermissionBoundary, UseCase, execute, execute_with_policy},
    },
};

#[utoipa::path(
    delete,
    tag = "Event",
    path = "/api/v1/user/events/{event_id}",
    summary = "Delete an event (admin only)",
    params(
        ("event_id" = ID, Path, description = "The id of the event to delete"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn delete_event_admin_controller(
    Extension(account): Extension<Account>,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;
    let user = account_can_modify_user(&account, &e.user_id, &ctx).await?;

    let usecase = DeleteEventUseCase {
        user,
        event_id: e.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| Json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

#[utoipa::path(
    delete,
    tag = "Event",
    path = "/api/v1/events/{event_id}",
    summary = "Delete an event (user only)",
    params(
        ("event_id" = ID, Path, description = "The id of the event to delete"),
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn delete_event_controller(
    Extension((user, policy)): Extension<(User, Policy)>,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let usecase = DeleteEventUseCase {
        user,
        event_id: path_params.event_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| Json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct DeleteEventUseCase {
    pub user: User,
    pub event_id: ID,
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(ID),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::NotFound(event_id) => Self::NotFound(format!(
                "The calendar event with id: {}, was not found.",
                event_id
            )),
        }
    }
}

#[async_trait::async_trait]
impl UseCase for DeleteEventUseCase {
    type Response = CalendarEvent;

    type Error = UseCaseError;

    const NAME: &'static str = "DeleteEvent";

    // TODO: use only one db call
    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let event = ctx
            .repos
            .events
            .find(&self.event_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        let e = match event {
            Some(e) if e.user_id == self.user.id => e,
            _ => return Err(UseCaseError::NotFound(self.event_id.clone())),
        };

        if !APP_CONFIG.disable_reminders {
            self.delete_synced_events(&e, ctx).await;
        }

        ctx.repos
            .events
            .delete(&e.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;

        Ok(e)
    }
}

impl PermissionBoundary for DeleteEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteCalendarEvent]
    }
}

impl DeleteEventUseCase {
    pub async fn delete_synced_events(&self, e: &CalendarEvent, ctx: &NitteiContext) {
        let synced_events = match ctx.repos.event_synced.find_by_event(&e.id).await {
            Ok(synced_events) => synced_events,
            Err(e) => {
                error!("Unable to query synced events from repo: {:?}", e);
                return;
            }
        };

        let synced_outlook_events = synced_events
            .iter()
            .filter(|o_event| o_event.provider == IntegrationProvider::Outlook)
            .collect::<Vec<_>>();
        let synced_google_events = synced_events
            .iter()
            .filter(|g_event| g_event.provider == IntegrationProvider::Google)
            .collect::<Vec<_>>();

        if synced_google_events.is_empty() && synced_outlook_events.is_empty() {
            return;
        }

        let user = ctx.repos.users.find(&e.user_id).await;
        let user = match user {
            Ok(Some(u)) => u,
            Ok(None) => {
                error!("Unable to find user when deleting sync events");
                return;
            }
            Err(e) => {
                error!("Unable to find user when deleting sync events {:?}", e);
                return;
            }
        };

        if !synced_outlook_events.is_empty() {
            let provider = match OutlookCalendarProvider::new(&user, ctx).await {
                Ok(p) => p,
                Err(_) => {
                    error!("Unable to create outlook calendar provider");
                    return;
                }
            };
            for cal in synced_outlook_events {
                if provider
                    .delete_event(cal.ext_calendar_id.clone(), cal.ext_event_id.clone())
                    .await
                    .is_err()
                {
                    error!("Unable to delete external outlook calendar event");
                };
            }
        }

        if !synced_google_events.is_empty() {
            let provider = match GoogleCalendarProvider::new(&user, ctx).await {
                Ok(p) => p,
                Err(_) => {
                    error!("Unable to create google calendar provider");
                    return;
                }
            };
            for cal in synced_google_events {
                if provider
                    .delete_event(cal.ext_calendar_id.clone(), cal.ext_event_id.clone())
                    .await
                    .is_err()
                {
                    error!("Unable to delete google external calendar event");
                };
            }
        }
    }
}
