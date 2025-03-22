use axum::{
    Extension,
    Json,
    extract::Path,
    http::HeaderMap,
};
use nittei_api_structs::get_user_by_external_id::*;
use nittei_domain::{Account, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn get_user_by_external_id_controller(
    headers: HeaderMap,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

    let usecase = GetUserByExternalIdUseCase {
        account,
        external_id: path_params.external_id.clone(),
    };
    execute(usecase, &ctx)
        .await
        .map(|usecase_res| Json(APIResponse::new(usecase_res.user)))
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

#[async_trait::async_trait]
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
