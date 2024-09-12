use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::update_user::*;
use nittei_domain::{Metadata, User, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn update_user_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    mut path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let usecase = UpdateUserUseCase {
        account_id: account.id,
        user_id: std::mem::take(&mut path.user_id),
        metadata: body.0.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct UpdateUserUseCase {
    pub account_id: ID,
    pub user_id: ID,
    pub metadata: Option<Metadata>,
}

#[derive(Debug)]
pub struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UseCaseError {
    StorageError,
    UserNotFound(ID),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::UserNotFound(id) => {
                Self::Conflict(format!("A user with id {} was not found", id))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateUserUseCase {
    type Response = UseCaseRes;
    type Error = UseCaseError;

    const NAME: &'static str = "UpdateUser";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let mut user = match ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account_id)
            .await
        {
            Ok(Some(user)) => user,
            Ok(None) => return Err(UseCaseError::UserNotFound(self.user_id.clone())),
            Err(_) => return Err(UseCaseError::StorageError),
        };

        if let Some(metadata) = &self.metadata {
            user.metadata = metadata.clone();
        }

        ctx.repos
            .users
            .save(&user)
            .await
            .map(|_| UseCaseRes { user })
            .map_err(|_| UseCaseError::StorageError)
    }
}
