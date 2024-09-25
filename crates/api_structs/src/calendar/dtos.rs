use nittei_domain::{Calendar, CalendarSettings, Metadata, Weekday, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Deserialize, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarDTO {
    pub id: ID,
    pub user_id: ID,
    pub settings: CalendarSettingsDTO,
    #[ts(type = "Record<string, string>")]
    pub metadata: Metadata,
}

#[derive(Debug, Deserialize, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarSettingsDTO {
    pub week_start: Weekday,
    pub timezone: String,
}

impl CalendarDTO {
    pub fn new(calendar: Calendar) -> Self {
        Self {
            id: calendar.id.clone(),
            user_id: calendar.user_id.clone(),
            settings: CalendarSettingsDTO::new(&calendar.settings),
            metadata: calendar.metadata,
        }
    }
}

impl CalendarSettingsDTO {
    pub fn new(settings: &CalendarSettings) -> Self {
        Self {
            week_start: settings.week_start,
            timezone: settings.timezone.to_string(),
        }
    }
}
