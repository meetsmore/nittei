use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::get_user_by_external_id::*;
use nittei_domain::{Account, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn get_user_by_external_id_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = GetUserByExternalIdUseCase {
        account,
        external_id: path_params.external_id.clone(),
    };
    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct GetUserByExternalIdUseCase {
    account: Account,
    external_id: String,
}

#[derive(Debug)]
struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UseCaseError {
    InternalError,
    UserNotFound(String),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::UserNotFound(id) => {
                Self::NotFound(format!("A user with external_id: {}, was not found.", id))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetUserByExternalIdUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "GetUserByExternalId";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let user = match ctx.repos.users.get_by_external_id(&self.external_id).await {
            Ok(Some(u)) if u.account_id == self.account.id => u,
            Ok(_) => return Err(UseCaseError::UserNotFound(self.external_id.clone())),
            Err(_) => return Err(UseCaseError::InternalError),
        };

        Ok(UseCaseRes { user })
    }
}
