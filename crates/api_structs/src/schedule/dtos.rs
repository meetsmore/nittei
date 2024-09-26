use nittei_domain::{Metadata, Schedule, ScheduleRule, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
/// A schedule is a set of rules that define when a service is available
pub struct ScheduleDTO {
    /// UUID of the schedule
    pub id: ID,
    /// UUID of the user that owns the schedule
    pub user_id: ID,
    /// Array of rules for this schedule
    pub rules: Vec<ScheduleRule>,
    /// Timezone (e.g. "America/New_York")
    pub timezone: String,
    /// Metadata (e.g. {"key": "value"})
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
