use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::update_service::*;
use nittei_domain::{ID, Service, ServiceMultiPersonOptions};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn update_service_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    mut path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = UpdateServiceUseCase {
        account_id: account.id,
        service_id: std::mem::take(&mut path.service_id),
        metadata: body.metadata,
        multi_person: body.multi_person,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.service)))
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

#[async_trait::async_trait]
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
