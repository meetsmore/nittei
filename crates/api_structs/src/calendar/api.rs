use nittei_domain::{Calendar, EventInstance, Tz, Weekday, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    dtos::{CalendarDTO, EventWithInstancesDTO},
    helpers::deserialize_uuids_list::deserialize_stringified_uuids_list,
};

/// Calendar object
#[derive(Deserialize, Serialize, TS)]
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

    #[derive(Deserialize)]
    pub struct QueryParams {
        pub key: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// API response for getting calendars by user
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetCalendarsByUserAPIResponse")]
    pub struct APIResponse {
        /// List of calendars
        pub calendars: Vec<CalendarDTO>,
    }

    impl APIResponse {
        pub fn new(calendars: Vec<Calendar>) -> Self {
            Self {
                calendars: calendars.into_iter().map(CalendarDTO::new).collect(),
            }
        }
    }
}

pub mod create_calendar {
    use nittei_domain::Metadata;

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// Request body for creating a calendar
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateCalendarRequestBody")]
    pub struct RequestBody {
        /// Timezone for the calendar (e.g. "America/New_York")
        #[ts(type = "string")]
        pub timezone: Tz,
        /// Weekday for the calendar
        /// Default is Monday
        #[serde(default = "default_weekday")]
        #[ts(optional, as = "Option<_>")]
        pub week_start: Weekday,
        #[ts(optional)]
        pub name: Option<String>,
        #[ts(optional)]
        pub key: Option<String>,
        /// Optional metadata (e.g. {"key": "value"})
        #[ts(optional, type = "Record<string, string>")]
        pub metadata: Option<Metadata>,
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
    #[ts(export, rename = "AddSyncCalendarPathParams")]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// Request body for adding a sync calendar
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "AddSyncCalendarRequestBody")]
    pub struct RequestBody {
        /// Integration provider
        /// E.g. Google, Outlook, etc.
        pub provider: IntegrationProvider,
        /// Calendar UUID to sync to
        pub calendar_id: ID,
        /// External calendar ID
        pub ext_calendar_id: String,
    }

    pub type APIResponse = String;
}

pub mod remove_sync_calendar {
    use nittei_domain::IntegrationProvider;

    use super::*;

    #[derive(Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "RemoveSyncCalendarPathParams")]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// Request body for removing a sync calendar
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "RemoveSyncCalendarRequestBody")]
    pub struct RequestBody {
        /// Integration provider
        /// E.g. Google, Outlook, etc.
        pub provider: IntegrationProvider,
        /// Calendar UUID to stop syncing to
        pub calendar_id: ID,
        /// External calendar ID
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
    #[derive(Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetCalendarEventsAPIResponse")]
    pub struct APIResponse {
        /// Calendar's data
        pub calendar: CalendarDTO,
        /// Events with their instances (occurrences)
        pub events: Vec<EventWithInstancesDTO>,
    }

    impl APIResponse {
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

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetCalendarsByMetaAPIResponse")]
    pub struct APIResponse {
        pub calendars: Vec<CalendarDTO>,
    }

    impl APIResponse {
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

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetGoogleCalendarsAPIResponse")]
    pub struct APIResponse {
        pub calendars: Vec<GoogleCalendarListEntry>,
    }

    impl APIResponse {
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

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetOutlookCalendarsAPIResponse")]
    pub struct APIResponse {
        pub calendars: Vec<OutlookCalendar>,
    }

    impl APIResponse {
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
    #[ts(export, rename = "GetUserFreeBusyQueryParams")]
    pub struct QueryParams {
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
    }

    /// API response for getting user free/busy
    #[derive(Debug, Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetUserFreeBusyAPIResponse")]
    pub struct APIResponse {
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
    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "MultipleFreeBusyRequestBody")]
    pub struct RequestBody {
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
    #[derive(Debug, Serialize, Deserialize, TS)]
    #[ts(export, rename = "MultipleFreeBusyAPIResponse")]
    pub struct APIResponse(pub HashMap<ID, Vec<EventInstance>>);
}

pub mod update_calendar {
    use nittei_domain::{Metadata, Weekday};

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub calendar_id: ID,
    }

    /// Request body for updating a calendar's settings
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "UpdateCalendarSettings")]
    pub struct CalendarSettings {
        /// Optional weekday for the calendar
        #[serde(default)]
        #[ts(optional)]
        pub week_start: Option<Weekday>,
        /// Optional timezone for the calendar (e.g. "America/New_York")
        #[ts(type = "string", optional)]
        pub timezone: Option<Tz>,
    }

    /// Request body for updating a calendar
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "UpdateCalendarRequestBody")]
    pub struct RequestBody {
        /// Calendar settings
        pub settings: CalendarSettings,
        // Name of the calendar
        pub name: Option<String>,
        /// Optional metadata (e.g. {"key": "value"})
        #[serde(default)]
        #[ts(optional, type = "Record<string, string>")]
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = CalendarResponse;
}
