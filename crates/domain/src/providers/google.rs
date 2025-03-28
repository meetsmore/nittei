use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum GoogleCalendarAccessRole {
    Owner,
    Writer,
    Reader,
    FreeBusyReader,
}

#[derive(Debug, Deserialize, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GoogleCalendarListEntry {
    pub id: String,
    pub access_role: GoogleCalendarAccessRole,
    pub summary: String,
    pub summary_override: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub time_zone: Option<String>,
    pub color_id: Option<String>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub hidden: Option<bool>,
    pub selected: Option<bool>,
    pub primary: Option<bool>,
    pub deleted: Option<bool>,
}
