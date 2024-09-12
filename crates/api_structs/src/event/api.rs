use nittei_domain::{CalendarEvent, CalendarEventReminder, EventInstance, RRuleOptions, ID};
use serde::{Deserialize, Serialize};

use crate::dtos::CalendarEventDTO;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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
    use nittei_domain::Metadata;

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub calendar_id: ID,
        pub start_time: DateTime<Utc>,
        pub duration: i64,
        #[serde(default)]
        pub busy: Option<bool>,
        #[serde(default)]
        pub recurrence: Option<RRuleOptions>,
        #[serde(default)]
        pub reminders: Vec<CalendarEventReminder>,
        #[serde(default)]
        pub service_id: Option<ID>,
        #[serde(default)]
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

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
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

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        #[serde(default, deserialize_with = "deserialize_stringified_uuids_list")]
        pub calendar_ids: Option<Vec<ID>>,
        pub start_time: DateTime<Utc>,
        pub end_time: DateTime<Utc>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
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

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
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

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        #[serde(default)]
        pub start_time: Option<DateTime<Utc>>,
        #[serde(default)]
        pub duration: Option<i64>,
        #[serde(default)]
        pub busy: Option<bool>,
        #[serde(default)]
        pub recurrence: Option<RRuleOptions>,
        #[serde(default)]
        pub service_id: Option<ID>,
        #[serde(default)]
        pub exdates: Option<Vec<DateTime<Utc>>>,
        #[serde(default)]
        pub reminders: Option<Vec<CalendarEventReminder>>,
        #[serde(default)]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
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
