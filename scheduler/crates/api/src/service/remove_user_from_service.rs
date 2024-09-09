use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::remove_user_from_service::*;
use nittei_domain::{Account, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn remove_user_from_service_controller(
    http_req: HttpRequest,
    mut path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = RemoveUserFromServiceUseCase {
        account,
        service_id: std::mem::take(&mut path.service_id),
        user_id: std::mem::take(&mut path.user_id),
    };

    execute(usecase, &ctx)
        .await
        .map(|_usecase_res| HttpResponse::Ok().json(APIResponse::from("User removed from service")))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct RemoveUserFromServiceUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
}

#[derive(Debug)]
struct UseCaseRes {}

#[derive(Debug)]
enum UseCaseError {
    InternalError,
    ServiceNotFound,
    UserNotFound,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::ServiceNotFound => {
                Self::NotFound("The requested service was not found".to_string())
            }
            UseCaseError::UserNotFound => {
                Self::NotFound("The specified user was not found in the service".to_string())
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for RemoveUserFromServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "RemoveUserFromService";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let service = match ctx.repos.services.find(&self.service_id).await {
            Ok(Some(service)) if service.account_id == self.account.id => service,
            Ok(_) => return Err(UseCaseError::ServiceNotFound),
            Err(_) => return Err(UseCaseError::InternalError),
        };

        ctx.repos
            .service_users
            .delete(&service.id, &self.user_id)
            .await
            .map(|_| UseCaseRes {})
            .map_err(|_| UseCaseError::UserNotFound)
    }
}
