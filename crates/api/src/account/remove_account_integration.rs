use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::remove_account_integration::{APIResponse, PathParams};
use nittei_domain::{Account, IntegrationProvider};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{UseCase, execute},
    },
};

pub async fn remove_account_integration_controller(
    http_req: HttpRequest,
    mut path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = RemoveAccountIntegrationUseCase {
        account,
        provider: std::mem::take(&mut path.provider),
    };

    execute(usecase, &ctx)
        .await
        .map(|_| {
            HttpResponse::Ok().json(APIResponse::from(
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

#[async_trait::async_trait(?Send)]
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
