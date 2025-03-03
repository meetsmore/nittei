use nittei_domain::{Calendar, CalendarSettings, ID, Weekday};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Calendar object
#[derive(Debug, Deserialize, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarDTO {
    /// UUID of the calendar
    pub id: ID,

    /// UUID of the user that owns the calendar
    pub user_id: ID,

    /// Name of the calendar (optional)
    #[ts(optional)]
    pub name: Option<String>,

    /// Key of the calendar (optional)
    /// When defined, this is unique per user
    #[ts(optional)]
    pub key: Option<String>,

    /// Calendar settings
    pub settings: CalendarSettingsDTO,

    /// Metadata (e.g. {"key": "value"})
    #[ts(optional)]
    pub metadata: Option<serde_json::Value>,
}

/// Calendar settings
#[derive(Debug, Deserialize, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarSettingsDTO {
    /// Week start day
    pub week_start: Weekday,
    /// Timezone (e.g. "America/New_York")
    pub timezone: String,
}

impl CalendarDTO {
    pub fn new(calendar: Calendar) -> Self {
        Self {
            id: calendar.id.clone(),
            user_id: calendar.user_id.clone(),
            name: calendar.name,
            key: calendar.key,
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
