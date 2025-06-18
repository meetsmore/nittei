use axum::{Extension, Json, extract::Path};
use nittei_api_structs::{dtos::CalendarEventDTO, get_event_by_external_id::*};
use nittei_domain::{Account, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::usecase::{UseCase, execute},
};

#[utoipa::path(
    get,
    tag = "Event",
    path = "/api/v1/user/events/external_id/{external_id}",
    summary = "Get an event by its external id (admin only)",
    params(
        ("external_id" = String, Path, description = "The external id of the event to get"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = GetEventsByExternalIdAPIResponse)
    )
)]
pub async fn get_event_by_external_id_admin_controller(
    Extension(account): Extension<Account>,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<GetEventsByExternalIdAPIResponse>, NitteiError> {
    let usecase = GetEventByExternalIdUseCase {
        account_id: account.id,
        external_id: path_params.external_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|events| Json(GetEventsByExternalIdAPIResponse::new(events)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetEventByExternalIdUseCase {
    pub external_id: String,
    pub account_id: ID,
}

#[derive(Debug)]
pub enum UseCaseError {
    InternalError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for GetEventByExternalIdUseCase {
    type Response = Vec<CalendarEventDTO>;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEventsByExternalId";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let events = ctx
            .repos
            .events
            .get_by_external_id(&self.account_id, &self.external_id)
            .await
            .map_err(|e| {
                tracing::error!("[get_event_by_external_id] Error getting events: {:?}", e);
                UseCaseError::InternalError
            })?;

        let events_as_dto: Vec<CalendarEventDTO> =
            events.into_iter().map(CalendarEventDTO::new).collect();

        Ok(events_as_dto)
    }
}
