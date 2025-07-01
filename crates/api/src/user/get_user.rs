use axum::{Extension, Json, extract::Path};
use nittei_api_structs::get_user::*;
use nittei_domain::{Account, ID, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::usecase::{UseCase, execute},
};

#[utoipa::path(
    get,
    tag = "User",
    path = "/api/v1/user/{user_id}",
    summary = "Get a user (admin only)",
    params(
        ("user_id" = ID, Path, description = "The id of the user to get"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn get_user_controller(
    Extension(account): Extension<Account>,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let usecase = GetUserUseCase {
        account,
        user_id: path_params.user_id.clone(),
    };
    execute(usecase, &ctx)
        .await
        .map(|usecase_res| Json(APIResponse::new(usecase_res.user)))
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
                Self::NotFound(format!("A user with id: {id}, was not found."))
            }
        }
    }
}

#[async_trait::async_trait]
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
