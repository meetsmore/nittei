use axum::{Extension, Json, http::StatusCode};
use axum_valid::Valid;
use futures::future::{self, try_join};
use nittei_api_structs::delete_many_events::DeleteManyEventsRequestBody;
use nittei_domain::{Account, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::Permission,
        usecase::{PermissionBoundary, UseCase, execute},
    },
};

#[utoipa::path(
    post,
    tag = "Event",
    path = "/api/v1/user/events/delete_many",
    summary = "Delete many events (admin only)",
    security(
        ("api_key" = [])
    ),
    request_body(
        content = DeleteManyEventsRequestBody,
    ),
    responses(
        (status = 200)
    )
)]
pub async fn delete_many_events_admin_controller(
    Extension(account): Extension<Account>,
    Extension(ctx): Extension<NitteiContext>,
    body: Valid<Json<DeleteManyEventsRequestBody>>,
) -> Result<StatusCode, NitteiError> {
    let usecase = DeleteManyEventsUseCase {
        account_uid: account.id,
        event_ids: body.event_ids.clone(),
        external_ids: body.external_ids.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|_| StatusCode::OK)
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct DeleteManyEventsUseCase {
    account_uid: ID,
    event_ids: Option<Vec<ID>>,
    external_ids: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum UseCaseError {
    BadRequest,
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::BadRequest => Self::BadClientData("No event ids provided".to_string()),
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for DeleteManyEventsUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "DeleteManyEvents";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        // If both event_ids and external_ids are None or if both are empty, return an error
        if self.event_ids.is_none() && self.external_ids.is_none() {
            tracing::warn!("[delete_many_events] No event ids or external ids provided");
            return Err(UseCaseError::BadRequest);
        }

        // If both event_ids and external_ids exist, but are empty, return an error
        if self.event_ids.as_ref().map(|v| v.len()).unwrap_or(1) == 0
            && self.external_ids.as_ref().map(|v| v.len()).unwrap_or(1) == 0
        {
            tracing::warn!("[delete_many_events] No event ids or external ids provided");
            return Err(UseCaseError::BadRequest);
        }

        // Find events by ids (it isn't awaited here, but instead in the try_join below)
        let events_by_ids = if let Some(ids) = &self.event_ids
            && !ids.is_empty()
        {
            ctx.repos.events.find_many(ids.as_slice())
        } else {
            Box::pin(future::ok(Vec::new())) // Creates an already-resolved future
        };

        // Find events by external ids (it isn't awaited here, but instead in the try_join below)
        let events_by_external_ids = if let Some(ids) = &self.external_ids
            && !ids.is_empty()
        {
            ctx.repos
                .events
                .find_many_by_external_ids(&self.account_uid, ids.as_slice())
        } else {
            Box::pin(future::ok(Vec::new())) // Another already-resolved future
        };

        // Join the two futures and wait for them to complete
        // This will return an error if any of the futures fail
        let (events_by_ids, events_by_external_ids) =
            try_join(events_by_ids, events_by_external_ids)
                .await
                .map_err(|e| {
                    tracing::error!("[delete_many_events] Error finding events: {:?}", e);
                    UseCaseError::StorageError
                })?;

        // Check if any of the events are from another account
        let event_from_other_account = events_by_ids
            .iter()
            .chain(events_by_external_ids.iter())
            .find(|event| event.account_id != self.account_uid);

        // If we found an event from another account, return an error
        if let Some(bad_event) = event_from_other_account {
            tracing::warn!(
                "[delete_many_events] Account {} is not allowed to delete events from account {}",
                self.account_uid,
                bad_event.account_id
            );
            return Err(UseCaseError::BadRequest);
        }

        // Merge event ids and remove duplicates
        let event_ids_to_delete = events_by_ids
            .iter()
            .chain(events_by_external_ids.iter())
            .map(|event| event.id.clone())
            .collect::<std::collections::HashSet<ID>>()
            .into_iter()
            .collect::<Vec<ID>>();

        ctx.repos
            .events
            .delete_many(event_ids_to_delete.as_slice())
            .await
            .map_err(|e| {
                tracing::error!("[delete_many_events] Error deleting events: {:?}", e);
                UseCaseError::StorageError
            })?;

        Ok(())
    }
}

impl PermissionBoundary for DeleteManyEventsUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteCalendarEvent]
    }
}
