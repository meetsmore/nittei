use axum::{Extension, Json, extract::Path};
use chrono_tz::Tz;
use nittei_api_structs::update_schedule::*;
use nittei_domain::{Account, ID, Schedule, ScheduleRule, User};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Permission, Policy, account_can_modify_schedule},
        usecase::{PermissionBoundary, UseCase, execute, execute_with_policy},
    },
};

pub async fn update_schedule_admin_controller(
    Extension(account): Extension<Account>,
    path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
    body: Json<RequestBody>,
) -> Result<Json<APIResponse>, NitteiError> {
    let schedule = account_can_modify_schedule(&account, &path.schedule_id, &ctx).await?;

    let mut body = body.0;
    let usecase = UpdateScheduleUseCase {
        user_id: schedule.user_id,
        schedule_id: schedule.id,
        timezone: body.timezone,
        rules: body.rules.take(),
        metadata: body.metadata.take(),
    };

    execute(usecase, &ctx)
        .await
        .map(|res| Json(APIResponse::new(res.schedule)))
        .map_err(NitteiError::from)
}

pub async fn update_schedule_controller(
    Extension((user, policy)): Extension<(User, Policy)>,
    Extension(ctx): Extension<NitteiContext>,
    path: Path<PathParams>,
    body: Json<RequestBody>,
) -> Result<Json<APIResponse>, NitteiError> {
    let mut path = path.0;
    let mut body = body.0;
    let usecase = UpdateScheduleUseCase {
        user_id: user.id,
        schedule_id: std::mem::take(&mut path.schedule_id),
        timezone: body.timezone,
        rules: body.rules.take(),
        metadata: body.metadata.take(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|res| Json(APIResponse::new(res.schedule)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct UpdateScheduleUseCase {
    pub user_id: ID,
    pub schedule_id: ID,
    pub timezone: Option<Tz>,
    pub rules: Option<Vec<ScheduleRule>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
enum UseCaseError {
    ScheduleNotFound(ID),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::ScheduleNotFound(schedule_id) => Self::NotFound(format!(
                "The schedule with id: {schedule_id}, was not found."
            )),
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[derive(Debug)]
struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait]
impl UseCase for UpdateScheduleUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "UpdateSchedule";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let mut schedule = match ctx.repos.schedules.find(&self.schedule_id).await {
            Ok(Some(cal)) if cal.user_id == self.user_id => cal,
            Ok(_) => return Err(UseCaseError::ScheduleNotFound(self.schedule_id.clone())),
            Err(_) => {
                return Err(UseCaseError::StorageError);
            }
        };

        if let Some(tz) = self.timezone {
            schedule.timezone = tz;
        };
        if let Some(rules) = &self.rules {
            schedule.set_rules(rules);
        }

        if self.metadata.is_some() {
            schedule.metadata = self.metadata.clone();
        }

        let repo_res = ctx.repos.schedules.save(&schedule).await;
        match repo_res {
            Ok(_) => Ok(UseCaseRes { schedule }),
            Err(_) => Err(UseCaseError::StorageError),
        }
    }
}

impl PermissionBoundary for UpdateScheduleUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateSchedule]
    }
}
