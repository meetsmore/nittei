use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::delete_schedule::*;
use nittei_domain::{ID, Schedule};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Permission, account_can_modify_schedule, protect_account_route, protect_route},
        usecase::{PermissionBoundary, UseCase, execute, execute_with_policy},
    },
};

pub async fn delete_schedule_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let schedule = account_can_modify_schedule(&account, &path.schedule_id, &ctx).await?;

    let usecase = DeleteScheduleUseCase {
        user_id: schedule.user_id,
        schedule_id: schedule.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|schedule| HttpResponse::Ok().json(APIResponse::new(schedule)))
        .map_err(NitteiError::from)
}

pub async fn delete_schedule_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteScheduleUseCase {
        user_id: user.id,
        schedule_id: path.schedule_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|schedule| HttpResponse::Ok().json(APIResponse::new(schedule)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(ID),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::NotFound(schedule_id) => Self::NotFound(format!(
                "The schedule with id: {}, was not found.",
                schedule_id
            )),
        }
    }
}

#[derive(Debug)]
pub struct DeleteScheduleUseCase {
    schedule_id: ID,
    user_id: ID,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteScheduleUseCase {
    type Response = Schedule;

    type Error = UseCaseError;

    const NAME: &'static str = "DeleteSchedule";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let schedule = ctx
            .repos
            .schedules
            .find(&self.schedule_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        match schedule {
            Some(schedule) if schedule.user_id == self.user_id => {
                ctx.repos
                    .schedules
                    .delete(&schedule.id)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                Ok(schedule)
            }
            _ => Err(UseCaseError::NotFound(self.schedule_id.clone())),
        }
    }
}

impl PermissionBoundary for DeleteScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteSchedule]
    }
}
