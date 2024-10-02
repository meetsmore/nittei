use nittei_domain::{CalendarEvent, CalendarEventReminder, EventInstance, RRuleOptions, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dtos::CalendarEventDTO;

/// Calendar event response object
#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarEventResponse {
    /// Calendar event retrieved
    pub event: CalendarEventDTO,
}

impl CalendarEventResponse {
    pub fn new(event: CalendarEvent) -> Self {
        Self {
            event: CalendarEventDTO::new(event),
        }
    }
}

pub mod create_event {
    use chrono::{DateTime, Utc};
    use nittei_domain::{CalendarEventStatus, Metadata};

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// Request body for creating an event
    #[derive(Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateEventRequestBody")]
    pub struct RequestBody {
        /// UUID of the calendar where the event will be created
        pub calendar_id: ID,

        /// Optional title of the event
        #[serde(default)]
        #[ts(optional)]
        pub title: Option<String>,

        /// Optional description of the event
        #[serde(default)]
        #[ts(optional)]
        pub description: Option<String>,

        /// Optional parent event ID
        /// This is useful for external applications that need to link Nittei's events to their own data models
        #[serde(default)]
        #[ts(optional)]
        pub parent_id: Option<String>,

        /// Optional location of the event
        #[serde(default)]
        #[ts(optional)]
        pub location: Option<String>,

        /// Optional status of the event
        /// Default is "Tentative"
        #[serde(default)]
        #[ts(optional, as = "Option<_>")]
        pub status: CalendarEventStatus,

        /// Optional flag to indicate if the event is an all day event
        /// Default is false
        #[serde(default)]
        #[ts(optional)]
        pub all_day: Option<bool>,

        /// Start time of the event (UTC)
        #[ts(type = "Date")]
        pub start_time: DateTime<Utc>,

        /// Duration of the event in minutes
        #[ts(type = "number")]
        pub duration: i64,

        /// Optional flag to indicate if the event is busy
        /// Default is false
        #[serde(default)]
        #[ts(optional)]
        pub busy: Option<bool>,

        /// Optional recurrence rule
        #[serde(default)]
        #[ts(optional)]
        pub recurrence: Option<RRuleOptions>,

        /// Optional list of reminders
        #[serde(default)]
        #[ts(optional, as = "Option<_>")]
        pub reminders: Vec<CalendarEventReminder>,

        /// Optional service UUID
        /// This is automatically set when the event is created from a service
        #[serde(default)]
        #[ts(optional)]
        pub service_id: Option<ID>,

        /// Optional metadata (e.g. {"key": "value"})
        #[serde(default)]
        #[ts(optional, type = "Record<string, string>")]
        pub metadata: Option<Metadata>,
    }

    pub type APIResponse = CalendarEventResponse;
}

pub mod delete_event {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: ID,
    }

    pub type APIResponse = CalendarEventResponse;
}

pub mod get_event_instances {

    use chrono::{DateTime, Utc};

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: ID,
    }
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub start_time: DateTime<Utc>,
        pub end_time: DateTime<Utc>,
    }

    /// API response for getting event instances
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventInstancesAPIResponse")]
    pub struct APIResponse {
        /// Calendar event
        pub event: CalendarEventDTO,
        /// List of event instances (occurrences)
        pub instances: Vec<EventInstance>,
    }

    impl APIResponse {
        pub fn new(event: CalendarEvent, instances: Vec<EventInstance>) -> Self {
            Self {
                event: CalendarEventDTO::new(event),
                instances,
            }
        }
    }
}

pub mod get_event {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: ID,
    }

    pub type APIResponse = CalendarEventResponse;
}

pub mod get_events_by_calendars {
    use chrono::{DateTime, Utc};
    use nittei_domain::EventWithInstances;

    use super::*;
    use crate::{
        dtos::EventWithInstancesDTO,
        helpers::deserialize_uuids_list::deserialize_stringified_uuids_list,
    };

    /// Query parameters for getting events by calendars
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventsByCalendarsQueryParams")]
    pub struct QueryParams {
        /// Optional list of calendar UUIDs
        /// If not provided, all calendars will be used
        #[serde(default, deserialize_with = "deserialize_stringified_uuids_list")]
        pub calendar_ids: Option<Vec<ID>>,

        /// Start time of the interval for getting the events (UTC)
        #[ts(type = "Date")]
        pub start_time: DateTime<Utc>,

        /// End time of the interval for getting the events (UTC)
        #[ts(type = "Date")]
        pub end_time: DateTime<Utc>,
    }

    /// API response for getting events by calendars
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventsByCalendarsAPIResponse")]
    pub struct APIResponse {
        /// List of calendar events retrieved
        pub events: Vec<EventWithInstancesDTO>,
    }

    impl APIResponse {
        pub fn new(events: Vec<EventWithInstances>) -> Self {
            Self {
                events: events
                    .into_iter()
                    .map(|e| EventWithInstancesDTO::new(e.event, e.instances))
                    .collect(),
            }
        }
    }
}

pub mod get_events_by_meta {
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

    /// API response for getting events by metadata
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventsByMetaAPIResponse")]
    pub struct APIResponse {
        /// List of calendar events retrieved
        pub events: Vec<CalendarEventDTO>,
    }

    impl APIResponse {
        pub fn new(events: Vec<CalendarEvent>) -> Self {
            Self {
                events: events.into_iter().map(CalendarEventDTO::new).collect(),
            }
        }
    }
}

pub mod update_event {
    use chrono::{DateTime, Utc};
    use nittei_domain::Metadata;

    use super::*;

    /// Request body for updating an event
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "UpdateEventRequestBody")]
    pub struct RequestBody {
        /// Optional start time of the event (UTC)
        #[serde(default)]
        #[ts(optional, type = "Date")]
        pub start_time: Option<DateTime<Utc>>,

        /// Optional duration of the event in minutes
        #[serde(default)]
        #[ts(optional, type = "number")]
        pub duration: Option<i64>,

        /// Optional busy flag
        #[serde(default)]
        #[ts(optional)]
        pub busy: Option<bool>,

        /// Optional new recurrence rule
        #[serde(default)]
        #[ts(optional)]
        pub recurrence: Option<RRuleOptions>,

        /// Optional service UUID
        #[serde(default)]
        #[ts(optional)]
        pub service_id: Option<ID>,

        /// Optional list of exclusion dates for the recurrence rule
        #[serde(default)]
        #[ts(optional, type = "Array<Date>")]
        pub exdates: Option<Vec<DateTime<Utc>>>,

        /// Optional list of reminders
        #[serde(default)]
        #[ts(optional)]
        pub reminders: Option<Vec<CalendarEventReminder>>,

        /// Optional metadata (e.g. {"key": "value"})
        #[serde(default)]
        #[ts(optional, type = "Record<string, string>")]
        pub metadata: Option<Metadata>,
    }

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_id: ID,
    }

    pub type APIResponse = CalendarEventResponse;
}

pub mod send_event_reminders {
    use super::*;

    #[derive(Debug)]
    pub struct AccountEventReminder {
        pub event: CalendarEvent,
        pub identifier: String,
    }

    /// Account event reminders DTO
    #[derive(Debug, Clone, Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AccountEventRemindersDTO {
        /// Calendar event
        event: CalendarEventDTO,
        /// Identifier of the reminder
        identifier: String,
    }

    impl AccountEventRemindersDTO {
        pub fn new(account_event_reminder: AccountEventReminder) -> Self {
            Self {
                event: CalendarEventDTO::new(account_event_reminder.event),
                identifier: account_event_reminder.identifier,
            }
        }
    }

    #[derive(Debug)]
    pub struct AccountReminders {
        pub reminders: Vec<AccountEventReminder>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AccountRemindersDTO {
        reminders: Vec<AccountEventRemindersDTO>,
    }

    impl AccountRemindersDTO {
        pub fn new(acc_reminders: AccountReminders) -> Self {
            Self {
                reminders: acc_reminders
                    .reminders
                    .into_iter()
                    .map(AccountEventRemindersDTO::new)
                    .collect(),
            }
        }
    }
}
