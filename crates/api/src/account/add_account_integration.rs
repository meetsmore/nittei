use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::add_account_integration::{APIResponse, AddAccountIntegrationRequestBody};
use nittei_domain::{Account, AccountIntegration, IntegrationProvider};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    put,
    tag = "Account",
    path = "/api/v1/account/integration",
    summary = "Add an integration to an account",
    security(
        ("api_key" = [])
    ),
    request_body(
        content = AddAccountIntegrationRequestBody,
    ),
    responses(
        (status = 200, description = "The integration was added successfully", body = APIResponse)
    )
)]
pub async fn add_account_integration_controller(
    http_req: HttpRequest,
    body: actix_web_validator::Json<AddAccountIntegrationRequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = AddAccountIntegrationUseCase {
        account,
        client_id: body.client_id,
        client_secret: body.client_secret,
        redirect_uri: body.redirect_uri,
        provider: body.provider,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().json(APIResponse::from("Integration enabled for account")))
        .map_err(NitteiError::from)
}

#[derive(Debug, Clone)]
pub struct AddAccountIntegrationUseCase {
    pub account: Account,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub provider: IntegrationProvider,
}

impl From<AddAccountIntegrationUseCase> for AccountIntegration {
    fn from(e: AddAccountIntegrationUseCase) -> Self {
        Self {
            account_id: e.account.id,
            client_id: e.client_id,
            client_secret: e.client_secret,
            redirect_uri: e.redirect_uri,
            provider: e.provider,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UseCaseError {
    StorageError,
    IntegrationAlreadyExists,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::IntegrationAlreadyExists => {
                Self::Conflict("Account already has an integration for that provider".into())
            }
        }
    }
}

impl From<anyhow::Error> for UseCaseError {
    fn from(_: anyhow::Error) -> Self {
        UseCaseError::StorageError
    }
}

#[async_trait::async_trait]
impl UseCase for AddAccountIntegrationUseCase {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "AddAccountIntegration";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        // TODO: check if it is possible to validate client id or client secret

        let acc_integrations = ctx
            .repos
            .account_integrations
            .find(&self.account.id)
            .await?;
        if acc_integrations.iter().any(|i| i.provider == self.provider) {
            return Err(UseCaseError::IntegrationAlreadyExists);
        }

        ctx.repos
            .account_integrations
            .insert(&self.clone().into())
            .await?;

        Ok(())
    }
}
