use axum::{Extension, Json, extract::Path, http::StatusCode};
use chrono_tz::Tz;
use nittei_api_structs::create_schedule::*;
use nittei_domain::{Account, ID, Schedule, ScheduleRule, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Permission, Policy, account_can_modify_user},
        usecase::{PermissionBoundary, UseCase, execute, execute_with_policy},
    },
};

pub async fn create_schedule_admin_controller(
    Extension(account): Extension<Account>,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
    body_params: Json<RequestBody>,
) -> Result<(StatusCode, Json<APIResponse>), NitteiError> {
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let mut body_params = body_params.0;
    let usecase = CreateScheduleUseCase {
        user_id: user.id,
        account_id: account.id,
        timezone: body_params.timezone,
        rules: body_params.rules.take(),
        metadata: body_params.metadata.take(),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| (StatusCode::CREATED, Json(APIResponse::new(res.schedule))))
        .map_err(NitteiError::from)
}

pub async fn create_schedule_controller(
    Extension((user, policy)): Extension<(User, Policy)>,
    Extension(ctx): Extension<NitteiContext>,
    body_params: Json<RequestBody>,
) -> Result<(StatusCode, Json<APIResponse>), NitteiError> {
    let mut body_params = body_params.0;
    let usecase = CreateScheduleUseCase {
        user_id: user.id,
        account_id: user.account_id,
        timezone: body_params.timezone,
        rules: body_params.rules.take(),
        metadata: body_params.metadata.take(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|res| (StatusCode::CREATED, Json(APIResponse::new(res.schedule))))
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
                Self::NotFound(format!("The user with id: {user_id}, was not found."))
            }
        }
    }
}

#[derive(Debug)]
struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait]
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
