use axum::{Json, extract::State, http::StatusCode};
use axum_valid::Valid;
use nittei_api_structs::create_account::{APIResponse, RequestBody};
use nittei_domain::Account;
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::usecase::{UseCase, execute},
};

pub async fn create_account_controller(
    State(ctx): State<NitteiContext>,
    body: Valid<Json<RequestBody>>,
) -> Result<(StatusCode, Json<APIResponse>), NitteiError> {
    let usecase = CreateAccountUseCase {
        code: body.0.code.clone(),
    };
    execute(usecase, &ctx)
        .await
        .map(|account| (StatusCode::CREATED, Json(APIResponse::new(account))))
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

#[async_trait::async_trait(?Send)]
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
