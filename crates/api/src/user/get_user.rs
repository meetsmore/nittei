use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::get_user::*;
use nittei_domain::{Account, User, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn get_user_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = GetUserUseCase {
        account,
        user_id: path_params.user_id.clone(),
    };
    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct GetUserUseCase {
    account: Account,
    user_id: ID,
}

#[derive(Debug)]
struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
enum UseCaseError {
    InternalError,
    UserNotFound(ID),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::UserNotFound(id) => {
                Self::NotFound(format!("A user with id: {}, was not found.", id))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetUserUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "GetUser";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let user = match ctx.repos.users.find(&self.user_id).await {
            Ok(Some(u)) if u.account_id == self.account.id => u,
            Ok(_) => return Err(UseCaseError::UserNotFound(self.user_id.clone())),
            Err(_) => return Err(UseCaseError::InternalError),
        };

        Ok(UseCaseRes { user })
    }
}
