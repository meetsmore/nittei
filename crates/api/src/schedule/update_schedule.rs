use actix_web::{HttpRequest, HttpResponse, web};
use chrono_tz::Tz;
use nittei_api_structs::update_schedule::*;
use nittei_domain::{ID, Schedule, ScheduleRule};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Permission, account_can_modify_schedule, protect_admin_route, protect_route},
        usecase::{PermissionBoundary, UseCase, execute, execute_with_policy},
    },
};

pub async fn update_schedule_admin_controller(
    http_req: HttpRequest,
    path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;
    let schedule = account_can_modify_schedule(&account, &path.schedule_id, &ctx).await?;

    let body = body.0;
    let usecase = UpdateScheduleUseCase {
        user_id: schedule.user_id,
        schedule_id: schedule.id,
        timezone: body.timezone,
        rules: body.rules,
        metadata: body.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.schedule)))
        .map_err(NitteiError::from)
}

pub async fn update_schedule_controller(
    http_req: HttpRequest,
    ctx: web::Data<NitteiContext>,
    mut path: web::Path<PathParams>,
    body: web::Json<RequestBody>,
) -> Result<HttpResponse, NitteiError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = UpdateScheduleUseCase {
        user_id: user.id,
        schedule_id: std::mem::take(&mut path.schedule_id),
        timezone: body.timezone,
        rules: body.rules,
        metadata: body.metadata,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|res| HttpResponse::Ok().json(APIResponse::new(res.schedule)))
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
