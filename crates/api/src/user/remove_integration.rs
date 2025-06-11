use axum::{Extension, Json, extract::Path, http::HeaderMap};
use nittei_api_structs::remove_integration::*;
use nittei_domain::{Account, ID, IntegrationProvider, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_user, protect_route},
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    delete,
    tag = "User",
    path = "/api/v1/user/{user_id}/oauth/{provider}",
    summary = "Remove an integration (admin only)",
    params(
        ("user_id" = ID, Path, description = "The id of the user to remove the integration from"),
        ("provider" = IntegrationProvider, Path, description = "The provider of the integration to remove"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn remove_integration_admin_controller(
    Extension(account): Extension<Account>,
    mut path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let user = account_can_modify_user(&account, &path.user_id, &ctx).await?;

    let usecase = OAuthIntegrationUseCase {
        user,
        provider: std::mem::take(&mut path.provider),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| Json(APIResponse::new(res.user)))
        .map_err(NitteiError::from)
}

#[utoipa::path(
    delete,
    tag = "User",
    path = "/api/v1/me/oauth/{provider}",
    summary = "Remove an integration",
    params(
        ("provider" = IntegrationProvider, Path, description = "The provider of the integration to remove"),
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn remove_integration_controller(
    headers: HeaderMap,
    mut path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let (user, _) = protect_route(&headers, &ctx).await?;

    let usecase = OAuthIntegrationUseCase {
        user,
        provider: std::mem::take(&mut path.provider),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| Json(APIResponse::new(res.user)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct OAuthIntegrationUseCase {
    pub user: User,
    pub provider: IntegrationProvider,
}

#[derive(Debug)]
pub struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UseCaseError {
    StorageError,
    IntegrationNotFound,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::IntegrationNotFound => {
                Self::NotFound("Did not find an integration between that user and provider".into())
            }
        }
    }
}

#[async_trait::async_trait]
impl UseCase for OAuthIntegrationUseCase {
    type Response = UseCaseRes;
    type Error = UseCaseError;

    const NAME: &'static str = "RemoveIntegration";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let user_integrations = ctx
            .repos
            .user_integrations
            .find(&self.user.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        if !user_integrations
            .into_iter()
            .any(|i| i.provider == self.provider)
        {
            return Err(UseCaseError::IntegrationNotFound);
        };

        ctx.repos
            .user_integrations
            .delete(&self.user.id, self.provider.clone())
            .await
            .map(|_| UseCaseRes {
                user: self.user.clone(),
            })
            .map_err(|_| UseCaseError::StorageError)
    }
}
