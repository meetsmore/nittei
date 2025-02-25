use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::{dtos::CalendarEventDTO, get_event_by_external_id::*};
use nittei_domain::ID;
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{UseCase, execute},
    },
};

pub async fn get_event_by_external_id_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = GetEventByExternalIdUseCase {
        account_id: account.id,
        external_id: path_params.external_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|events| HttpResponse::Ok().json(APIResponse::new(events)))
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

#[async_trait::async_trait(?Send)]
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
            .map_err(|_| UseCaseError::InternalError)?;

        let events_as_dto: Vec<CalendarEventDTO> =
            events.into_iter().map(CalendarEventDTO::new).collect();

        Ok(events_as_dto)
    }
}
