use axum::{Extension, Json, http::StatusCode};
use nittei_api_structs::create_service::*;
use nittei_domain::{Account, Service, ServiceMultiPersonOptions};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::usecase::{UseCase, execute},
};

pub async fn create_service_controller(
    Extension(account): Extension<Account>,
    Extension(ctx): Extension<NitteiContext>,
    body: Json<RequestBody>,
) -> Result<(StatusCode, Json<APIResponse>), NitteiError> {
    let mut body = body.0;
    let usecase = CreateServiceUseCase {
        account,
        metadata: body.metadata.take(),
        multi_person: body.multi_person.take().unwrap_or_default(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            (
                StatusCode::CREATED,
                Json(APIResponse::new(usecase_res.service)),
            )
        })
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct CreateServiceUseCase {
    account: Account,
    multi_person: ServiceMultiPersonOptions,
    metadata: Option<serde_json::Value>,
}
#[derive(Debug)]
struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseError {
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for CreateServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateService";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let mut service = Service::new(self.account.id.clone());
        service.metadata = self.metadata.clone();
        service.multi_person = self.multi_person.clone();

        ctx.repos
            .services
            .insert(&service)
            .await
            .map(|_| UseCaseRes { service })
            .map_err(|_| UseCaseError::StorageError)
    }
}
