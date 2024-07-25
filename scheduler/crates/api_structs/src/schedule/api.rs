use nettu_scheduler_domain::{Schedule, Tz, ID};
use serde::{Deserialize, Serialize};

use crate::dtos::ScheduleDTO;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleResponse {
    pub schedule: ScheduleDTO,
}

impl ScheduleResponse {
    pub fn new(schedule: Schedule) -> Self {
        Self {
            schedule: ScheduleDTO::new(schedule),
        }
    }
}

pub mod create_schedule {
    use nettu_scheduler_domain::{Metadata, ScheduleRule};

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub timezone: Tz,
        pub rules: Option<Vec<ScheduleRule>>,
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod delete_schedule {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: ID,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod get_schedule {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: ID,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod update_schedule {
    use nettu_scheduler_domain::{Metadata, ScheduleRule};

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub schedule_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub timezone: Option<Tz>,
        pub rules: Option<Vec<ScheduleRule>>,
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = ScheduleResponse;
}

pub mod get_schedules_by_meta {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub key: String,
        pub value: String,
        #[serde(default)]
        pub skip: Option<usize>,
        pub limit: Option<usize>,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub schedules: Vec<ScheduleDTO>,
    }

    impl APIResponse {
        pub fn new(schedules: Vec<Schedule>) -> Self {
            Self {
                schedules: schedules.into_iter().map(ScheduleDTO::new).collect(),
            }
        }
    }
}
