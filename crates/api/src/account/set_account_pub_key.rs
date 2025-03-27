use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::set_account_pub_key::{APIResponse, SetAccountPubKeyRequestBody};
use nittei_domain::{Account, PEMKey};
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
    path = "/api/v1/account/pubkey",
    summary = "Set the public key for an account",
    security(
        ("api_key" = [])
    ),
    request_body(
        content = SetAccountPubKeyRequestBody,
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn set_account_pub_key_controller(
    http_req: HttpRequest,
    ctx: web::Data<NitteiContext>,
    body: actix_web_validator::Json<SetAccountPubKeyRequestBody>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let usecase = SetAccountPubKeyUseCase {
        account,
        public_jwt_key: body.public_jwt_key.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|account| HttpResponse::Ok().json(APIResponse::new(account)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct SetAccountPubKeyUseCase {
    pub account: Account,
    pub public_jwt_key: Option<String>,
}

#[derive(Debug)]
enum UseCaseError {
    InvalidPemKey,
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InvalidPemKey => {
                Self::BadClientData("Malformed public pem key provided".into())
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for SetAccountPubKeyUseCase {
    type Response = Account;

    type Error = UseCaseError;

    const NAME: &'static str = "SetAccountPublicKey";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let key = if let Some(key) = &self.public_jwt_key {
            match PEMKey::new(key.clone()) {
                Ok(key) => Some(key),
                Err(_) => return Err(UseCaseError::InvalidPemKey),
            }
        } else {
            None
        };
        self.account.set_public_jwt_key(key);

        match ctx.repos.accounts.save(&self.account).await {
            Ok(_) => Ok(self.account.clone()),
            Err(_) => Err(UseCaseError::StorageError),
        }
    }
}
