use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::remove_integration::*;
use nittei_domain::{IntegrationProvider, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_user, protect_account_route, protect_route},
        usecase::{execute, UseCase},
    },
};

pub async fn remove_integration_admin_controller(
    http_req: HttpRequest,
    mut path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path.user_id, &ctx).await?;

    let usecase = OAuthIntegrationUseCase {
        user,
        provider: std::mem::take(&mut path.provider),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.user)))
        .map_err(NitteiError::from)
}

pub async fn remove_integration_controller(
    http_req: HttpRequest,
    mut path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, _) = protect_route(&http_req, &ctx).await?;

    let usecase = OAuthIntegrationUseCase {
        user,
        provider: std::mem::take(&mut path.provider),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.user)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct OAuthIntegrationUseCase {
    pub user: User,
    pub provider: IntegrationProvider,
}

#[derive(Debug)]
pub struct UseCaseRes {
    pub user: User,
}

#[derive(Debug)]
pub enum UseCaseError {
    StorageError,
    IntegrationNotFound,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::IntegrationNotFound => {
                Self::NotFound("Did not find an integration between that user and provider".into())
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for OAuthIntegrationUseCase {
    type Response = UseCaseRes;
    type Error = UseCaseError;

    const NAME: &'static str = "RemoveIntegration";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let user_integrations = ctx
            .repos
            .user_integrations
            .find(&self.user.id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        if !user_integrations
            .into_iter()
            .any(|i| i.provider == self.provider)
        {
            return Err(UseCaseError::IntegrationNotFound);
        };

        ctx.repos
            .user_integrations
            .delete(&self.user.id, self.provider.clone())
            .await
            .map(|_| UseCaseRes {
                user: self.user.clone(),
            })
            .map_err(|_| UseCaseError::StorageError)
    }
}
