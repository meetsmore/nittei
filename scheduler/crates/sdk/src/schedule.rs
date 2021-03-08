use crate::{APIResponse, BaseClient};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::{ScheduleRule, ID};
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct ScheduleClient {
    base: Arc<BaseClient>,
}

pub struct CreateScheduleInput {
    pub timezone: String,
    pub rules: Option<Vec<ScheduleRule>>,
    pub user_id: ID,
}

pub struct UpdateScheduleInput {
    pub timezone: Option<String>,
    pub rules: Option<Vec<ScheduleRule>>,
    pub schedule_id: ID,
}

impl ScheduleClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn get(&self, schedule_id: String) -> APIResponse<get_schedule::APIResponse> {
        self.base
            .get(format!("user/schedule/{}", schedule_id), StatusCode::OK)
            .await
    }

    pub async fn delete(&self, schedule_id: String) -> APIResponse<delete_schedule::APIResponse> {
        self.base
            .delete(format!("user/schedule/{}", schedule_id), StatusCode::OK)
            .await
    }

    pub async fn update(
        &self,
        input: UpdateScheduleInput,
    ) -> APIResponse<update_schedule::APIResponse> {
        let body = update_schedule::RequestBody {
            timezone: input.timezone,
            rules: input.rules,
        };

        self.base
            .put(
                body,
                format!("user/schedule/{}", input.schedule_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn create(
        &self,
        input: CreateScheduleInput,
    ) -> APIResponse<create_schedule::APIResponse> {
        let body = create_schedule::RequestBody {
            timezone: input.timezone,
            rules: input.rules,
        };
        let path = create_schedule::PathParams {
            user_id: input.user_id,
        };

        self.base
            .post(
                body,
                format!("user/{}/schedule", path.user_id),
                StatusCode::CREATED,
            )
            .await
    }
}
