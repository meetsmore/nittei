use axum::{
    Extension,
    Json,
    http::{HeaderMap, StatusCode},
};
use axum_valid::Valid;
use futures::future::{self, try_join};
use nittei_api_structs::delete_many_events::DeleteManyEventsRequestBody;
use nittei_domain::ID;
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Permission, protect_admin_route},
        usecase::{PermissionBoundary, UseCase, execute},
    },
};

pub async fn delete_many_events_admin_controller(
    headers: HeaderMap,
    Extension(ctx): Extension<NitteiContext>,
    body: Valid<Json<DeleteManyEventsRequestBody>>,
) -> Result<StatusCode, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

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
        // If both event_ids and external_ids are None, return an error
        if self.event_ids.is_none() && self.external_ids.is_none() {
            return Err(UseCaseError::BadRequest);
        }

        let events_by_ids = if let Some(ids) = &self.event_ids {
            ctx.repos.events.find_many(ids.as_slice())
        } else {
            Box::pin(future::ok(Vec::new())) // Creates an already-resolved future
        };

        let events_by_external_ids = if let Some(ids) = &self.external_ids {
            ctx.repos
                .events
                .find_many_by_external_ids(&self.account_uid, ids.as_slice())
        } else {
            Box::pin(future::ok(Vec::new())) // Another already-resolved future
        };

        let (events_by_ids, events_by_external_ids) =
            try_join(events_by_ids, events_by_external_ids)
                .await
                .map_err(|_| UseCaseError::StorageError)?;

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
            .map_err(|_| UseCaseError::StorageError)?;

        Ok(())
    }
}

impl PermissionBoundary for DeleteManyEventsUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteCalendarEvent]
    }
}
