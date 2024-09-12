use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::set_account_webhook::{APIResponse, RequestBody};
use nittei_domain::Account;
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn set_account_webhook_controller(
    http_req: HttpRequest,
    ctx: web::Data<NitteiContext>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = SetAccountWebhookUseCase {
        account,
        webhook_url: Some(body.webhook_url.clone()),
    };

    execute(usecase, &ctx)
        .await
        .map(|account| HttpResponse::Ok().json(APIResponse::new(account)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct SetAccountWebhookUseCase {
    pub account: Account,
    pub webhook_url: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseError {
    InvalidURI(String),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InvalidURI(err) => {
                Self::BadClientData(format!("Invalid URI provided. Error message: {}", err))
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for SetAccountWebhookUseCase {
    type Response = Account;

    type Error = UseCaseError;

    const NAME: &'static str = "SetAccountWebhook";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let success = self
            .account
            .settings
            .set_webhook_url(self.webhook_url.clone());

        if !success {
            return Err(UseCaseError::InvalidURI(format!(
                "Malformed url or scheme is not https: {:?}",
                self.webhook_url
            )));
        }

        match ctx.repos.accounts.save(&self.account).await {
            Ok(_) => Ok(self.account.clone()),
            Err(_) => Err(UseCaseError::StorageError),
        }
    }
}

#[cfg(test)]
mod tests {

    use nittei_infra::setup_context;

    use super::*;

    #[actix_web::main]
    #[test]
    async fn it_rejects_invalid_webhook_url() {
        let ctx = setup_context().await.unwrap();
        let bad_uris = vec!["1", "", "test.zzcom", "test.com", "google.com"];
        for bad_uri in bad_uris {
            let mut use_case = SetAccountWebhookUseCase {
                webhook_url: Some(bad_uri.to_string()),
                account: Default::default(),
            };
            let res = use_case.execute(&ctx).await;
            assert!(res.is_err());
            if let Err(err) = res {
                assert_eq!(
                    err,
                    UseCaseError::InvalidURI(format!(
                        "Malformed url or scheme is not https: {:?}",
                        Some(bad_uri)
                    ))
                );
            }
        }
    }

    #[actix_web::main]
    #[test]
    async fn it_accepts_valid_webhook_url() {
        let ctx = setup_context().await.unwrap();

        let valid_uris = vec!["https://google.com", "https://google.com/v1/webhook"];
        for valid_uri in valid_uris {
            let mut use_case = SetAccountWebhookUseCase {
                webhook_url: Some(valid_uri.to_string()),
                account: Default::default(),
            };
            let res = use_case.execute(&ctx).await;
            assert!(res.is_ok());
        }
    }
}
