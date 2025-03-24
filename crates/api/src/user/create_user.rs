use actix_web::{HttpRequest, HttpResponse, web};
use futures::{FutureExt, try_join};
use nittei_api_structs::create_user::*;
use nittei_domain::{ID, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn create_user_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let usecase = CreateUserUseCase {
        account_id: account.id,
        metadata: body.0.metadata,
        external_id: body.0.external_id,
        user_id: body.0.user_id,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Created().json(APIResponse::new(usecase_res.user)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct CreateUserUseCase {
    pub account_id: ID,
    pub metadata: Option<serde_json::Value>,
    pub external_id: Option<String>,
    pub user_id: Option<ID>,
}

#[derive(Debug)]
pub struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UseCaseError {
    StorageError,
    UserAlreadyExists,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::UserAlreadyExists => Self::Conflict(
                "A user with that userId already exist. UserIds need to be unique.".into(),
            ),
        }
    }
}
#[async_trait::async_trait]
impl UseCase for CreateUserUseCase {
    type Response = UseCaseRes;
    type Error = UseCaseError;

    const NAME: &'static str = "CreateUser";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let mut user = User::new(self.account_id.clone(), self.user_id.clone());
        user.metadata = self.metadata.clone();
        user.external_id = self.external_id.clone();

        let find_user = ctx.repos.users.find(&user.id);
        let find_by_external_id = match &user.external_id {
            Some(external_id) => ctx.repos.users.get_by_external_id(external_id),
            None => async { Ok(None) }.boxed(), // Dummy future if there's no external ID
        };

        let (existing_user, existing_user_by_external_id) =
            try_join!(find_user, find_by_external_id).map_err(|_| UseCaseError::StorageError)?;

        if existing_user.is_some() || existing_user_by_external_id.is_some() {
            return Err(UseCaseError::UserAlreadyExists);
        }

        let res = ctx.repos.users.insert(&user).await;
        match res {
            Ok(_) => Ok(UseCaseRes { user }),
            Err(_) => Err(UseCaseError::StorageError),
        }
    }
}
