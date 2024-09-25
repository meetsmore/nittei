use nittei_domain::{Metadata, Schedule, ScheduleRule, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScheduleDTO {
    pub id: ID,
    pub user_id: ID,
    pub rules: Vec<ScheduleRule>,
    pub timezone: String,
    #[ts(type = "Record<string, string>")]
    pub metadata: Metadata,
}

impl ScheduleDTO {
    pub fn new(schedule: Schedule) -> Self {
        Self {
            id: schedule.id.clone(),
            user_id: schedule.user_id.clone(),
            rules: schedule.rules,
            timezone: schedule.timezone.to_string(),
            metadata: schedule.metadata,
        }
    }
}
