use nittei_domain::{CalendarEvent, CalendarEventReminder, EventInstance, RRuleOptions, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dtos::CalendarEventDTO;

#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarEventResponse {
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

    #[derive(Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateEventRequestBody")]
    pub struct RequestBody {
        pub calendar_id: ID,
        #[serde(default)]
        #[ts(optional)]
        pub title: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        pub description: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        pub parent_id: Option<String>,
        #[serde(default)]
        #[ts(optional)]
        pub location: Option<String>,
        #[serde(default)]
        #[ts(optional, as = "Option<_>")]
        pub status: CalendarEventStatus,

        #[serde(default)]
        #[ts(optional)]
        pub all_day: Option<bool>,
        #[ts(type = "Date")]
        pub start_time: DateTime<Utc>,
        #[ts(type = "number")]
        pub duration: i64,
        #[serde(default)]
        #[ts(optional)]
        pub busy: Option<bool>,

        #[serde(default)]
        #[ts(optional)]
        pub recurrence: Option<RRuleOptions>,
        #[serde(default)]
        #[ts(optional, as = "Option<_>")]
        pub reminders: Vec<CalendarEventReminder>,
        #[serde(default)]
        #[ts(optional)]
        pub service_id: Option<ID>,
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

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventInstancesAPIResponse")]
    pub struct APIResponse {
        pub event: CalendarEventDTO,
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
    use crate::helpers::deserialize_uuids_list::deserialize_stringified_uuids_list;

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventsByCalendarsQueryParams")]
    pub struct QueryParams {
        #[serde(default, deserialize_with = "deserialize_stringified_uuids_list")]
        pub calendar_ids: Option<Vec<ID>>,
        #[ts(type = "Date")]
        pub start_time: DateTime<Utc>,
        #[ts(type = "Date")]
        pub end_time: DateTime<Utc>,
    }

    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventsByCalendarsAPIResponse")]
    pub struct APIResponse {
        pub events: Vec<EventWithInstances>,
    }

    impl APIResponse {
        pub fn new(events: Vec<EventWithInstances>) -> Self {
            Self { events }
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

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventsByMetaAPIResponse")]
    pub struct APIResponse {
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

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "UpdateEventRequestBody")]
    pub struct RequestBody {
        #[serde(default)]
        #[ts(optional, type = "Date")]
        pub start_time: Option<DateTime<Utc>>,
        #[serde(default)]
        #[ts(optional, type = "number")]
        pub duration: Option<i64>,
        #[serde(default)]
        #[ts(optional)]
        pub busy: Option<bool>,
        #[serde(default)]
        #[ts(optional)]
        pub recurrence: Option<RRuleOptions>,
        #[serde(default)]
        #[ts(optional)]
        pub service_id: Option<ID>,
        #[serde(default)]
        #[ts(optional, type = "Array<Date>")]
        pub exdates: Option<Vec<DateTime<Utc>>>,
        #[serde(default)]
        #[ts(optional)]
        pub reminders: Option<Vec<CalendarEventReminder>>,
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

    #[derive(Debug, Clone, Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AccountEventRemindersDTO {
        event: CalendarEventDTO,
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
