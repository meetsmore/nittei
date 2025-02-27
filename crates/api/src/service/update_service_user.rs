use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::update_service_user::*;
use nittei_domain::{Account, ID, ServiceResource, TimePlan};
use nittei_infra::NitteiContext;

use super::add_user_to_service::{
    ServiceResourceUpdate,
    UpdateServiceResourceError,
    update_resource_values,
};
use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn update_service_user_controller(
    http_req: HttpRequest,
    mut body: web::Json<RequestBody>,
    mut path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let usecase = UpdateServiceUserUseCase {
        account,
        service_id: std::mem::take(&mut path.service_id),
        user_id: std::mem::take(&mut path.user_id),
        availability: std::mem::take(&mut body.availability),
        buffer_after: body.buffer_after,
        buffer_before: body.buffer_before,
        closest_booking_time: body.closest_booking_time,
        furthest_booking_time: body.furthest_booking_time,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(APIResponse::new(usecase_res.user)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct UpdateServiceUserUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
    pub availability: Option<TimePlan>,
    pub buffer_after: Option<i64>,
    pub buffer_before: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

#[derive(Debug)]
struct UseCaseRes {
    pub user: ServiceResource,
}

#[derive(Debug)]
enum UseCaseError {
    StorageError,
    ServiceNotFound,
    UserNotFound,
    InvalidValue(UpdateServiceResourceError),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::ServiceNotFound => {
                Self::NotFound("The requested service was not found".into())
            }
            UseCaseError::UserNotFound => Self::NotFound("The specified user was not found".into()),
            UseCaseError::InvalidValue(e) => e.to_nittei_error(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateServiceUserUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "UpdateServiceUser";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let _service = match ctx.repos.services.find(&self.service_id).await {
            Ok(Some(service)) if service.account_id == self.account.id => service,
            Ok(_) => return Err(UseCaseError::ServiceNotFound),
            Err(_) => return Err(UseCaseError::StorageError),
        };

        let mut user_resource = match ctx
            .repos
            .service_users
            .find(&self.service_id, &self.user_id)
            .await
        {
            Ok(Some(res)) => res,
            Ok(None) => return Err(UseCaseError::UserNotFound),
            Err(_) => return Err(UseCaseError::StorageError),
        };

        update_resource_values(
            &mut user_resource,
            &ServiceResourceUpdate {
                availability: self.availability.clone(),
                buffer_after: self.buffer_after,
                buffer_before: self.buffer_before,
                closest_booking_time: self.closest_booking_time,
                furthest_booking_time: self.furthest_booking_time,
            },
            ctx,
        )
        .await
        .map_err(UseCaseError::InvalidValue)?;

        ctx.repos
            .service_users
            .save(&user_resource)
            .await
            .map(|_| UseCaseRes {
                user: user_resource,
            })
            .map_err(|_| UseCaseError::StorageError)
    }
}
