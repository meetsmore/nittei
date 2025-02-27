use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use nittei_api_structs::get_schedule::*;
use nittei_domain::{Schedule, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_schedule, protect_account_route, protect_route},
        usecase::{execute, UseCase},
    },
};

pub async fn get_schedule_admin_controller(
    headers: HeaderMap,
    path: Path<PathParams>,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_account_route(&headers, &ctx).await?;
    let schedule = account_can_modify_schedule(&account, &path.schedule_id, &ctx).await?;

    let usecase = GetScheduleUseCase {
        schedule_id: schedule.id,
    };

    execute(usecase, &ctx)
        .await
        .map(|schedule| Json(APIResponse::new(schedule)))
        .map_err(NitteiError::from)
}

pub async fn get_schedule_controller(
    headers: HeaderMap,
    req: Path<PathParams>,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let (_user, _policy) = protect_route(&headers, &ctx).await?;

    let usecase = GetScheduleUseCase {
        schedule_id: req.schedule_id.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|schedule| Json(APIResponse::new(schedule)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct GetScheduleUseCase {
    pub schedule_id: ID,
}

#[derive(Debug)]
enum UseCaseError {
    InternalError,
    NotFound(ID),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::NotFound(schedule_id) => Self::NotFound(format!(
                "The schedule with id: {}, was not found.",
                schedule_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetScheduleUseCase {
    type Response = Schedule;

    type Error = UseCaseError;

    const NAME: &'static str = "GetSchedule";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let schedule = ctx
            .repos
            .schedules
            .find(&self.schedule_id)
            .await
            .map_err(|_| UseCaseError::InternalError)?;
        match schedule {
            Some(schedule) => Ok(schedule),
            _ => Err(UseCaseError::NotFound(self.schedule_id.clone())),
        }
    }
}
