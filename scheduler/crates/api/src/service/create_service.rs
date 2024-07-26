use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::create_service::*;
use nettu_scheduler_domain::{Account, Metadata, Service, ServiceMultiPersonOptions};
use nettu_scheduler_infra::NettuContext;

use crate::{
    error::NettuError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn create_service_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = CreateServiceUseCase {
        account,
        metadata: body.metadata.unwrap_or_default(),
        multi_person: body.multi_person.unwrap_or_default(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Created().json(APIResponse::new(usecase_res.service)))
        .map_err(NettuError::from)
}

#[derive(Debug)]
struct CreateServiceUseCase {
    account: Account,
    multi_person: ServiceMultiPersonOptions,
    metadata: Metadata,
}
#[derive(Debug)]
struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseError {
    StorageError,
}

impl From<UseCaseError> for NettuError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateService";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
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
