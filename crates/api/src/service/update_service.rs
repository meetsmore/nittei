use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use axum_valid::Valid;
use nittei_api_structs::update_service::*;
use nittei_domain::{Service, ServiceMultiPersonOptions, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn update_service_controller(
    headers: HeaderMap,
    body: Valid<Json<RequestBody>>,
    mut path: Path<PathParams>,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_account_route(&headers, &ctx).await?;

    let mut body = body.0;
    let usecase = UpdateServiceUseCase {
        account_id: account.id,
        service_id: std::mem::take(&mut path.service_id),
        metadata: body.metadata.take(),
        multi_person: body.multi_person.take(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| Json(APIResponse::new(usecase_res.service)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct UpdateServiceUseCase {
    account_id: ID,
    service_id: ID,
    metadata: Option<serde_json::Value>,
    multi_person: Option<ServiceMultiPersonOptions>,
}
#[derive(Debug)]
struct UseCaseRes {
    pub service: Service,
}

#[derive(Debug)]
enum UseCaseError {
    StorageError,
    ServiceNotFound(ID),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::ServiceNotFound(id) => {
                Self::NotFound(format!("Service with id: {} was not found.", id))
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "UpdateService";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let mut service = match ctx.repos.services.find(&self.service_id).await {
            Ok(Some(service)) if service.account_id == self.account_id => service,
            Ok(_) => return Err(UseCaseError::ServiceNotFound(self.service_id.clone())),
            Err(_) => return Err(UseCaseError::StorageError),
        };

        if self.metadata.is_some() {
            service.metadata = self.metadata.clone();
        }
        if let Some(opts) = &self.multi_person {
            if let ServiceMultiPersonOptions::Group(new_count) = opts {
                if let ServiceMultiPersonOptions::Group(old_count) = &service.multi_person {
                    if new_count > old_count {
                        // Delete all calendar events for this service, because
                        // then it should be possible for more people to book
                        ctx.repos
                            .events
                            .delete_by_service(&service.id)
                            .await
                            .map_err(|_| UseCaseError::StorageError)?;
                    }
                }
            }
            service.multi_person = opts.clone();
        }

        ctx.repos
            .services
            .save(&service)
            .await
            .map(|_| UseCaseRes { service })
            .map_err(|_| UseCaseError::StorageError)
    }
}
