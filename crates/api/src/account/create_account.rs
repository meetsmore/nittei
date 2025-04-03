use actix_web::{HttpResponse, web};
use nittei_api_structs::create_account::{CreateAccountRequestBody, CreateAccountResponseBody};
use nittei_domain::Account;
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::usecase::{UseCase, execute},
};

#[utoipa::path(
    post,
    tag = "Account",
    path = "/api/v1/account",
    summary = "Create a new account",
    security(
        ("api_key" = [])
    ),
    request_body(
        content = CreateAccountRequestBody,
    ),
    responses(
        (status = 200, description = "The account was created successfully", body = CreateAccountResponseBody)
    )
)]
pub async fn create_account_controller(
    ctx: web::Data<NitteiContext>,
    body: actix_web_validator::Json<CreateAccountRequestBody>,
) -> Result<HttpResponse, NitteiError> {
    let usecase = CreateAccountUseCase { code: body.0.code };
    execute(usecase, &ctx)
        .await
        .map(|account| HttpResponse::Created().json(CreateAccountResponseBody::new(account)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct CreateAccountUseCase {
    code: String,
}

#[derive(Debug)]
enum UseCaseError {
    StorageError,
    InvalidCreateAccountCode,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InvalidCreateAccountCode => {
                Self::Unauthorized("Invalid code provided".into())
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for CreateAccountUseCase {
    type Response = Account;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateAccount";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        if self.code != ctx.config.create_account_secret_code {
            return Err(UseCaseError::InvalidCreateAccountCode);
        }
        let account = Account::new();
        let res = ctx.repos.accounts.insert(&account).await;

        res.map(|_| account).map_err(|_| UseCaseError::StorageError)
    }
}
