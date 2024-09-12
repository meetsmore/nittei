use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::get_service::*;
use nittei_domain::{Account, ServiceWithUsers, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn get_service_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = GetServiceUseCase {
        account,
        service_id: path_params.service_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.service)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct GetServiceUseCase {
    account: Account,
    service_id: ID,
}

#[derive(Debug)]
struct UseCaseRes {
    pub service: ServiceWithUsers,
}

#[derive(Debug)]
enum UseCaseError {
    InternalError,
    NotFound(ID),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::NotFound(id) => {
                Self::NotFound(format!("The service with id: {} was not found.", id))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;
    const NAME: &'static str = "GetService";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let res = ctx
            .repos
            .services
            .find_with_users(&self.service_id)
            .await
            .map_err(|_| UseCaseError::InternalError)?;
        match res {
            Some(service) if service.account_id == self.account.id => Ok(UseCaseRes { service }),
            _ => Err(UseCaseError::NotFound(self.service_id.clone())),
        }
    }
}
