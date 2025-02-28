use actix_web::{HttpRequest, HttpResponse, web};
use chrono_tz::Tz;
use nittei_api_structs::create_schedule::*;
use nittei_domain::{ID, Schedule, ScheduleRule};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Permission, account_can_modify_user, protect_admin_route, protect_route},
        usecase::{PermissionBoundary, UseCase, execute, execute_with_policy},
    },
};

pub async fn create_schedule_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body_params: web::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let usecase = CreateScheduleUseCase {
        user_id: user.id,
        account_id: account.id,
        timezone: body_params.0.timezone,
        rules: body_params.0.rules,
        metadata: body_params.0.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Created().json(APIResponse::new(res.schedule)))
        .map_err(NitteiError::from)
}

pub async fn create_schedule_controller(
    http_req: HttpRequest,
    body_params: web::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = CreateScheduleUseCase {
        user_id: user.id,
        account_id: user.account_id,
        timezone: body_params.0.timezone,
        rules: body_params.0.rules,
        metadata: body_params.0.metadata,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|res| HttpResponse::Created().json(APIResponse::new(res.schedule)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct CreateScheduleUseCase {
    pub user_id: ID,
    pub account_id: ID,
    pub timezone: Tz,
    pub rules: Option<Vec<ScheduleRule>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
enum UseCaseError {
    UserNotFound(ID),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::UserNotFound(user_id) => {
                Self::NotFound(format!("The user with id: {}, was not found.", user_id))
            }
        }
    }
}

#[derive(Debug)]
struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateScheduleUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateSchedule";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let user = ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?
            .ok_or_else(|| UseCaseError::UserNotFound(self.user_id.clone()))?;

        let mut schedule = Schedule::new(user.id, user.account_id, &self.timezone);
        if let Some(rules) = &self.rules {
            schedule.rules.clone_from(rules);
        }
        if self.metadata.is_some() {
            schedule.metadata = self.metadata.clone();
        }

        let res = ctx.repos.schedules.insert(&schedule).await;
        match res {
            Ok(_) => Ok(UseCaseRes { schedule }),
            Err(_) => Err(UseCaseError::StorageError),
        }
    }
}

impl PermissionBoundary for CreateScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateSchedule]
    }
}
