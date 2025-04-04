use axum::{Extension, Json, extract::Path, http::HeaderMap};
use nittei_api_structs::remove_account_integration::{APIResponse, PathParams};
use nittei_domain::{Account, IntegrationProvider};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    delete,
    tag = "Account",
    path = "/api/v1/account/integration/{provider}",
    summary = "Remove an integration from an account",
    params(
        ("provider" = IntegrationProvider, Path, description = "The provider of the integration to remove"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn remove_account_integration_controller(
    headers: HeaderMap,
    mut path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

    let usecase = RemoveAccountIntegrationUseCase {
        account,
        provider: std::mem::take(&mut path.provider),
    };

    execute(usecase, &ctx)
        .await
        .map(|_| {
            Json(APIResponse::from(
                "Provider integration removed from account",
            ))
        })
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct RemoveAccountIntegrationUseCase {
    pub account: Account,
    pub provider: IntegrationProvider,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseError {
    StorageError,
    IntegrationNotFound,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::IntegrationNotFound => Self::NotFound(
                "Did not find an integration between the given account and provider".into(),
            ),
        }
    }
}

impl From<anyhow::Error> for UseCaseError {
    fn from(_: anyhow::Error) -> Self {
        UseCaseError::StorageError
    }
}

#[async_trait::async_trait]
impl UseCase for RemoveAccountIntegrationUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "RemoveAccountIntegration";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let acc_integrations = ctx
            .repos
            .account_integrations
            .find(&self.account.id)
            .await?;
        if !acc_integrations.iter().any(|i| i.provider == self.provider) {
            return Err(UseCaseError::IntegrationNotFound);
        }

        ctx.repos
            .account_integrations
            .delete(&self.account.id, self.provider.clone())
            .await?;
        Ok(())
    }
}
