use axum::{Extension, Json, extract::Path};
use nittei_api_structs::delete_service::*;
use nittei_domain::{Account, ID, Service};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::usecase::{UseCase, execute},
};

pub async fn delete_service_controller(
    Extension(account): Extension<Account>,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let usecase = DeleteServiceUseCase {
        account,
        service_id: path_params.service_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| Json(APIResponse::new(usecase_res.service)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct DeleteServiceUseCase {
    account: Account,
    service_id: ID,
}

#[derive(Debug)]
struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseError {
    NotFound(ID),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(id) => {
                Self::NotFound(format!("The service with id: {id} was not found."))
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for DeleteServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "DeleteService";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let res = ctx
            .repos
            .services
            .find(&self.service_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        match res {
            Some(service) if service.account_id == self.account.id => {
                ctx.repos
                    .services
                    .delete(&self.service_id)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                Ok(UseCaseRes { service })
            }
            _ => Err(UseCaseError::NotFound(self.service_id.clone())),
        }
    }
}
