use nittei_domain::{Calendar, EventInstance, ID, Tz, Weekday};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    dtos::{CalendarDTO, EventWithInstancesDTO},
    helpers::deserialize_uuids_list::deserialize_stringified_uuids_list,
};

/// Calendar object
#[derive(Deserialize, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarResponse {
    /// Calendar retrieved
    pub calendar: CalendarDTO,
}

impl CalendarResponse {
    pub fn new(calendar: Calendar) -> Self {
        Self {
            calendar: CalendarDTO::new(calendar),
        }
    }
}

pub mod get_calendars_by_user {
    use super::*;

    #[derive(Deserialize, Validate)]
    pub struct QueryParams {
        #[validate(length(min = 1))]
        pub key: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// API response for getting calendars by user
    #[derive(Deserialize, Serialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct GetCalendarsByUserAPIResponse {
        /// List of calendars
        pub calendars: Vec<CalendarDTO>,
    }

    impl GetCalendarsByUserAPIResponse {
        pub fn new(calendars: Vec<Calendar>) -> Self {
            Self {
                calendars: calendars.into_iter().map(CalendarDTO::new).collect(),
            }
        }
    }
}

pub mod create_calendar {

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// Request body for creating a calendar
    #[derive(Deserialize, Serialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct CreateCalendarRequestBody {
        /// Timezone for the calendar (e.g. "America/New_York")
        #[ts(type = "string")]
        #[schema(value_type = Type::String)]
        pub timezone: Tz,
        /// Weekday for the calendar
        /// Default is Monday
        #[serde(default = "default_weekday")]
        #[ts(optional, as = "Option<_>")]
        #[schema(value_type = Type::String)]
        pub week_start: Weekday,

        /// Optional name for the calendar
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub name: Option<String>,

        /// Optional key for the calendar
        ///
        /// This allows to have 1 specific "type" of calendar per user
        /// And therefore, to target it more easily without listing all calendars
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub key: Option<String>,

        /// Optional metadata (e.g. {"key": "value"})
        #[ts(optional)]
        pub metadata: Option<serde_json::Value>,
    }

    pub type APIResponse = CalendarResponse;
}

fn default_weekday() -> Weekday {
    Weekday::Mon
}

pub mod add_sync_calendar {
    use nittei_domain::IntegrationProvider;

    use super::*;

    #[derive(Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AddSyncCalendarPathParams {
        pub user_id: ID,
    }

    /// Request body for adding a sync calendar
    #[derive(Deserialize, Serialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AddSyncCalendarRequestBody {
        /// Integration provider
        /// E.g. Google, Outlook, etc.
        pub provider: IntegrationProvider,

        /// Calendar UUID to sync to
        pub calendar_id: ID,

        /// External calendar ID
        #[validate(length(min = 1))]
        pub ext_calendar_id: String,
    }

    pub type APIResponse = String;
}

pub mod remove_sync_calendar {
    use nittei_domain::IntegrationProvider;

    use super::*;

    #[derive(Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct RemoveSyncCalendarPathParams {
        pub user_id: ID,
    }

    /// Request body for removing a sync calendar
    #[derive(Deserialize, Serialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct RemoveSyncCalendarRequestBody {
        /// Integration provider
        /// E.g. Google, Outlook, etc.
        pub provider: IntegrationProvider,

        /// Calendar UUID to stop syncing to
        pub calendar_id: ID,

        /// External calendar ID
        #[validate(length(min = 1))]
        pub ext_calendar_id: String,
    }

    pub type APIResponse = String;
}

pub mod delete_calendar {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub calendar_id: ID,
    }

    pub type APIResponse = CalendarResponse;
}

pub mod get_calendar_events {
    use chrono::{DateTime, Utc};
    use nittei_domain::EventWithInstances;

    use super::*;
    use crate::dtos::CalendarEventDTO;

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub calendar_id: ID,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub start_time: DateTime<Utc>,
        pub end_time: DateTime<Utc>,
    }

    /// API response for getting calendar events
    #[derive(Serialize, Deserialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct GetCalendarEventsAPIResponse {
        /// Calendar's data
        pub calendar: CalendarDTO,
        /// Events with their instances (occurrences)
        pub events: Vec<EventWithInstancesDTO>,
    }

    impl GetCalendarEventsAPIResponse {
        pub fn new(calendar: Calendar, events: Vec<EventWithInstances>) -> Self {
            Self {
                calendar: CalendarDTO::new(calendar),
                events: events
                    .into_iter()
                    .map(|e| EventWithInstancesDTO {
                        event: CalendarEventDTO::new(e.event),
                        instances: e.instances,
                    })
                    .collect(),
            }
        }
    }
}

pub mod get_calendar {
    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct PathParams {
        pub calendar_id: ID,
    }

    pub type APIResponse = CalendarResponse;
}

pub mod get_calendars_by_meta {
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

    #[derive(Deserialize, Serialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct GetCalendarsByMetaAPIResponse {
        pub calendars: Vec<CalendarDTO>,
    }

    impl GetCalendarsByMetaAPIResponse {
        pub fn new(calendars: Vec<Calendar>) -> Self {
            Self {
                calendars: calendars.into_iter().map(CalendarDTO::new).collect(),
            }
        }
    }
}

pub mod get_google_calendars {
    use nittei_domain::providers::google::{GoogleCalendarAccessRole, GoogleCalendarListEntry};

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub min_access_role: GoogleCalendarAccessRole,
    }

    #[derive(Deserialize, Serialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct GetGoogleCalendarsAPIResponse {
        pub calendars: Vec<GoogleCalendarListEntry>,
    }

    impl GetGoogleCalendarsAPIResponse {
        pub fn new(calendars: Vec<GoogleCalendarListEntry>) -> Self {
            Self { calendars }
        }
    }
}

pub mod get_outlook_calendars {
    use nittei_domain::providers::outlook::{OutlookCalendar, OutlookCalendarAccessRole};

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub min_access_role: OutlookCalendarAccessRole,
    }

    #[derive(Deserialize, Serialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct GetOutlookCalendarsAPIResponse {
        pub calendars: Vec<OutlookCalendar>,
    }

    impl GetOutlookCalendarsAPIResponse {
        pub fn new(calendars: Vec<OutlookCalendar>) -> Self {
            Self { calendars }
        }
    }
}

pub mod get_user_freebusy {
    use chrono::{DateTime, Utc};

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// Query parameters for getting user free/busy
    #[derive(Debug, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct GetUserFreeBusyQueryParams {
        /// Start time for the query (UTC)
        #[ts(type = "Date")]
        pub start_time: DateTime<Utc>,

        /// End time for the query (UTC)
        #[ts(type = "Date")]
        pub end_time: DateTime<Utc>,

        /// Optional list of calendar UUIDs to query
        /// If not provided, all calendars of the user will be queried
        #[serde(default, deserialize_with = "deserialize_stringified_uuids_list")]
        pub calendar_ids: Option<Vec<ID>>,

        /// Optional flag to include tentative events
        /// Default is false
        #[serde(default)]
        #[ts(optional)]
        pub include_tentative: Option<bool>,
    }

    /// API response for getting user free/busy
    #[derive(Debug, Serialize, Deserialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct GetUserFreeBusyAPIResponse {
        /// List of busy events
        pub busy: Vec<EventInstance>,
        /// UUID of the user
        pub user_id: String,
    }
}

pub mod multiple_freebusy {
    use std::collections::HashMap;

    use chrono::{DateTime, Utc};

    use super::*;

    /// Request body for getting multiple free/busy
    #[derive(Debug, Deserialize, Serialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct MultipleFreeBusyRequestBody {
        /// List of user UUIDs to query
        #[serde(default)]
        pub user_ids: Vec<ID>,

        /// Start time for the query (UTC)
        #[ts(type = "Date")]
        pub start_time: DateTime<Utc>,

        /// End time for the query (UTC)
        #[ts(type = "Date")]
        pub end_time: DateTime<Utc>,
    }

    /// API response for getting multiple free/busy
    /// HashMap<user_id, List of busy events>
    #[derive(Debug, Serialize, Deserialize, TS, ToSchema)]
    #[ts(export)]
    pub struct MultipleFreeBusyAPIResponse(pub HashMap<ID, Vec<EventInstance>>);
}

pub mod update_calendar {
    use nittei_domain::Weekday;

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub calendar_id: ID,
    }

    /// Request body for updating a calendar's settings
    #[derive(Deserialize, Serialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct UpdateCalendarSettings {
        /// Optional weekday for the calendar
        #[serde(default)]
        #[ts(optional)]
        #[schema(value_type = Type::String)]
        pub week_start: Option<Weekday>,

        /// Optional timezone for the calendar (e.g. "America/New_York")
        #[ts(type = "string", optional)]
        #[schema(value_type = Type::String)]
        pub timezone: Option<Tz>,
    }

    /// Request body for updating a calendar
    #[derive(Deserialize, Serialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct UpdateCalendarRequestBody {
        /// Calendar settings
        pub settings: UpdateCalendarSettings,

        /// Name of the calendar
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub name: Option<String>,

        /// Optional metadata (e.g. {"key": "value"})
        #[serde(default)]
        #[ts(optional)]
        pub metadata: Option<serde_json::Value>,
    }

    pub type APIResponse = CalendarResponse;
}
