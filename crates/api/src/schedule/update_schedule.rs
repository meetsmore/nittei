use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use axum_valid::Valid;
use chrono_tz::Tz;
use nittei_api_structs::update_schedule::*;
use nittei_domain::{Schedule, ScheduleRule, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_schedule, protect_account_route, protect_route, Permission},
        usecase::{execute, execute_with_policy, PermissionBoundary, UseCase},
    },
};

pub async fn update_schedule_admin_controller(
    headers: HeaderMap,
    path: Path<PathParams>,
    body: Valid<Json<RequestBody>>,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_account_route(&headers, &ctx).await?;
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
    headers: HeaderMap,
    State(ctx): State<NitteiContext>,
    mut path: Path<PathParams>,
    body: Valid<Json<RequestBody>>,
) -> Result<Json<APIResponse>, NitteiError> {
    let (user, policy) = protect_route(&headers, &ctx).await?;

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
                "The schedule with id: {}, was not found.",
                schedule_id
            )),
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[derive(Debug)]
struct UseCaseRes {
    pub schedule: Schedule,
}

#[async_trait::async_trait(?Send)]
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
