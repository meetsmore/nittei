use actix_web::{HttpRequest, HttpResponse, web};
use chrono::{DateTime, Utc};
use nittei_api_structs::remove_service_event_intend::*;
use nittei_domain::{Account, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{UseCase, execute},
    },
};

pub async fn remove_service_event_intend_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    mut path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let query = query_params.0;
    let usecase = RemoveServiceEventIntendUseCase {
        account,
        service_id: std::mem::take(&mut path_params.service_id),
        timestamp: query.timestamp,
    };

    execute(usecase, &ctx)
        .await
        .map(|_| HttpResponse::Ok().json(APIResponse::default()))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct RemoveServiceEventIntendUseCase {
    pub account: Account,
    pub service_id: ID,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
struct UseCaseRes {}

#[derive(Debug)]
enum UseCaseError {
    ServiceNotFound,
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::ServiceNotFound => {
                Self::NotFound("The requested service was not found".into())
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for RemoveServiceEventIntendUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "RemoveServiceEventIntend";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        match ctx.repos.services.find(&self.service_id).await {
            Ok(Some(s)) if s.account_id == self.account.id => (),
            Ok(_) => return Err(UseCaseError::ServiceNotFound),
            Err(_) => return Err(UseCaseError::StorageError),
        };
        ctx.repos
            .reservations
            .decrement(&self.service_id, self.timestamp)
            .await
            .map(|_| UseCaseRes {})
            .map_err(|_| UseCaseError::StorageError)
    }
}
