use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use axum_valid::Valid;
use nittei_api_structs::update_user::*;
use nittei_domain::{ID, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn update_user_controller(
    headers: HeaderMap,
    mut body: Valid<Json<RequestBody>>,
    mut path: Path<PathParams>,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

    let usecase = UpdateUserUseCase {
        account_id: account.id,
        external_id: body.0.external_id.take(),
        user_id: std::mem::take(&mut path.user_id),
        metadata: body.0.metadata.take(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| Json(APIResponse::new(usecase_res.user)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct UpdateUserUseCase {
    pub account_id: ID,
    pub external_id: Option<String>,
    pub user_id: ID,
    pub metadata: Option<serde_json::Value>,
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
            user.metadata = Some(metadata.clone());
        }

        if let Some(external_id) = &self.external_id {
            user.external_id = Some(external_id.clone());
        }

        ctx.repos
            .users
            .save(&user)
            .await
            .map(|_| UseCaseRes { user })
            .map_err(|_| UseCaseError::StorageError)
    }
}
