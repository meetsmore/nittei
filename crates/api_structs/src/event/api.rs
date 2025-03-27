use nittei_domain::{CalendarEvent, CalendarEventReminder, EventInstance, ID, RRuleOptions};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

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
    use nittei_domain::CalendarEventStatus;

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// Validate that recurring_event_id and original_start_time are both provided or both omitted
    fn validate_recurring_event_id_and_original_start_time(
        body: &RequestBody,
    ) -> Result<(), ValidationError> {
        if (body.recurring_event_id.is_some() && body.original_start_time.is_none())
            || (body.recurring_event_id.is_none() && body.original_start_time.is_some())
        {
            return Err(ValidationError::new(
                "Both recurring_event_id and original_start_time must be provided, or must be omitted",
            ));
        }
        Ok(())
    }

    /// Request body for creating an event
    #[derive(Serialize, Deserialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateEventRequestBody")]
    #[validate(schema(function = "validate_recurring_event_id_and_original_start_time"))]
    pub struct RequestBody {
        /// UUID of the calendar where the event will be created
        pub calendar_id: ID,

        /// Optional title of the event
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub title: Option<String>,

        /// Optional description of the event
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub description: Option<String>,

        /// Optional type of the event
        /// e.g. "meeting", "reminder", "birthday"
        /// Default is None
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub event_type: Option<String>,

        /// Optional parent event ID
        /// This is useful for external applications that need to link Nittei's events to a wider data model (e.g. a project, an order, etc.)
        /// Example: If the event is a meeting, the parent ID could be the project ID (ObjectId, UUID or any other string)
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub external_parent_id: Option<String>,

        /// Optional external event ID
        /// This is useful for external applications that need to link Nittei's events to their own data models
        /// Example: If the event is a meeting, the external ID could be the meeting ID in the external system
        ///
        /// Note that nothing prevents multiple events from having the same external ID
        /// This can also be a way to link events together
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub external_id: Option<String>,

        /// Optional location of the event
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
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

        /// Duration of the event in milliseconds
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

        /// Optional list of exclusion dates for the recurrence rule
        #[serde(default)]
        #[ts(optional, type = "Array<Date>")]
        pub exdates: Option<Vec<DateTime<Utc>>>,

        /// Optional recurring event ID
        /// This is the ID of the recurring event that this event is part of
        /// Default is None
        #[serde(default)]
        #[ts(optional)]
        pub recurring_event_id: Option<ID>,

        /// Optional original start time of the event
        /// This is the original start time of the event before it was moved (only for recurring events)
        /// Default is None
        #[serde(default)]
        #[ts(optional, type = "Date")]
        pub original_start_time: Option<DateTime<Utc>>,

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
        #[ts(optional)]
        pub metadata: Option<serde_json::Value>,

        /// Optional created date
        /// Defaults to the current date and time
        #[serde(default)]
        #[ts(optional, type = "Date")]
        pub created: Option<DateTime<Utc>>,

        /// Optional updated date
        /// Defaults to the current date and time
        #[serde(default)]
        #[ts(optional, type = "Date")]
        pub updated: Option<DateTime<Utc>>,
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

pub mod delete_many_events {
    use super::*;

    /// Request body for deleting many events (by event_ids and/or by external_ids)
    #[derive(Serialize, Deserialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "DeleteManyEventsRequestBody")]
    pub struct DeleteManyEventsRequestBody {
        /// List of event IDs to delete
        #[validate(length(min = 1))]
        #[ts(optional)]
        pub event_ids: Option<Vec<ID>>,

        /// List of events' external IDs to delete
        #[validate(length(min = 1))]
        #[ts(optional)]
        pub external_ids: Option<Vec<String>>,
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

pub mod get_event_by_external_id {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub external_id: String,
    }

    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventsByExternalIdAPIResponse")]
    pub struct APIResponse {
        /// Calendar events retrieved
        pub events: Vec<CalendarEventDTO>,
    }

    /// API response for getting events by calendars
    impl APIResponse {
        pub fn new(events: Vec<CalendarEventDTO>) -> Self {
            Self { events }
        }
    }
}

pub mod get_events_by_calendars {
    use chrono::{DateTime, Utc};
    use nittei_domain::EventWithInstances;

    use super::*;
    use crate::{
        dtos::EventWithInstancesDTO,
        helpers::deserialize_uuids_list::deserialize_stringified_uuids_list,
    };

    /// Path parameters for getting events by calendars
    #[derive(Deserialize, Serialize)]
    pub struct PathParams {
        /// ID of the user to fetch the events
        pub user_id: ID,
    }

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

pub mod get_events_for_users_in_time_range {
    use chrono::{DateTime, Utc};
    use nittei_domain::EventWithInstances;

    use super::*;
    use crate::dtos::EventWithInstancesDTO;

    /// Body for getting events for users in a time range
    #[derive(Deserialize, Serialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventsForUsersInTimeSpanBody")]
    pub struct RequestBody {
        /// List of user IDs
        #[validate(length(min = 1))]
        pub user_ids: Vec<ID>,

        /// Start time of the interval for getting the events (UTC)
        #[ts(type = "Date")]
        pub start_time: DateTime<Utc>,

        /// End time of the interval for getting the events (UTC)
        #[ts(type = "Date")]
        pub end_time: DateTime<Utc>,

        /// Generate instances of recurring events, default is false
        #[ts(optional)]
        pub generate_instances_for_recurring: Option<bool>,

        /// Include tentative events, default is false
        #[ts(optional)]
        pub include_tentative: Option<bool>,

        /// Include non-busy events, default is false
        #[ts(optional)]
        pub include_non_busy: Option<bool>,
    }

    /// API response for getting events by calendars
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetEventsForUsersInTimeSpanAPIResponse")]
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

pub mod search_events {
    use nittei_domain::{CalendarEventSort, DateTimeQuery, IDQuery, StringQuery};

    use super::*;

    /// Request body for searching events for one user
    #[derive(Deserialize, Serialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    #[ts(export, rename = "SearchEventsRequestBody")]
    pub struct RequestBody {
        /// Filter to use for searching events
        pub filter: RequestBodyFilter,

        /// Optional sort to use when searching events
        #[ts(optional)]
        pub sort: Option<CalendarEventSort>,

        /// Optional limit to use when searching events (u16)
        /// Default is 200
        #[ts(optional)]
        pub limit: Option<u16>,
    }

    /// Part of the Request body for searching events for a user
    /// This is the filter
    #[derive(Deserialize, Serialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    #[ts(
        export,
        rename = "SearchEventsRequestBodyFilter",
        rename_all = "camelCase"
    )]
    pub struct RequestBodyFilter {
        /// User ID
        pub user_id: ID,

        /// Optional query on event UUID(s)
        #[ts(optional)]
        pub event_uid: Option<IDQuery>,

        /// Optional list of calendar UUIDs
        /// If not provided, all calendars will be used
        #[validate(length(min = 1))]
        #[ts(optional)]
        pub calendar_ids: Option<Vec<ID>>,

        /// Optional query on external ID (which is a string as it's an ID from an external system)
        #[ts(optional)]
        pub external_id: Option<StringQuery>,

        /// Optional query on external parent ID (which is a string as it's an ID from an external system)
        #[ts(optional)]
        pub external_parent_id: Option<StringQuery>,

        /// Optional query on start time - "lower than or equal", or "great than or equal" (UTC)
        #[ts(optional)]
        pub start_time: Option<DateTimeQuery>,

        /// Optional query on end time - "lower than or equal", or "great than or equal" (UTC)
        #[ts(optional)]
        pub end_time: Option<DateTimeQuery>,

        /// Optional query on event type
        #[ts(optional)]
        pub event_type: Option<StringQuery>,

        /// Optional query on status
        #[ts(optional)]
        pub status: Option<StringQuery>,

        /// Optional query on the recurring event UID
        #[ts(optional)]
        pub recurring_event_uid: Option<IDQuery>,

        /// Optional query on original start time - "lower than or equal", or "great than or equal" (UTC)
        #[ts(optional)]
        pub original_start_time: Option<DateTimeQuery>,

        /// Optional filter on the recurrence (existence)
        #[ts(optional)]
        pub is_recurring: Option<bool>,

        /// Optional list of metadata key-value pairs
        #[ts(optional)]
        pub metadata: Option<serde_json::Value>,

        /// Optional query on created at - e.g. "lower than or equal", or "great than or equal" (UTC)
        #[ts(optional)]
        pub created_at: Option<DateTimeQuery>,

        /// Optional query on updated at - "lower than or equal", or "great than or equal" (UTC)
        #[ts(optional)]
        pub updated_at: Option<DateTimeQuery>,
    }

    /// API response for searching events for one user
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "SearchEventsAPIResponse")]
    pub struct APIResponse {
        /// List of calendar events retrieved
        pub events: Vec<CalendarEventDTO>,
    }

    impl APIResponse {
        pub fn new(events: Vec<CalendarEventDTO>) -> Self {
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
    use nittei_domain::CalendarEventStatus;

    use super::*;

    /// Validate that recurring_event_id and original_start_time are both provided or both omitted
    fn validate_recurring_event_id_and_original_start_time(
        body: &RequestBody,
    ) -> Result<(), ValidationError> {
        if (body.recurring_event_id.is_some() && body.original_start_time.is_none())
            || (body.recurring_event_id.is_none() && body.original_start_time.is_some())
        {
            return Err(ValidationError::new(
                "Both recurring_event_id and original_start_time must be provided, or must be omitted",
            ));
        }
        Ok(())
    }

    /// Request body for updating an event
    #[derive(Deserialize, Serialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "UpdateEventRequestBody")]
    #[validate(schema(function = "validate_recurring_event_id_and_original_start_time"))]
    pub struct RequestBody {
        /// Optional start time of the event (UTC)
        #[serde(default)]
        #[ts(optional, type = "Date")]
        pub start_time: Option<DateTime<Utc>>,

        /// Optional title of the event
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub title: Option<String>,

        /// Optional description of the event
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub description: Option<String>,

        /// Optional type of the event
        /// e.g. "meeting", "reminder", "birthday"
        /// Default is None
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub event_type: Option<String>,

        /// Optional parent event ID
        /// This is useful for external applications that need to link Nittei's events to a wider data model (e.g. a project, an order, etc.)
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub parent_id: Option<String>,

        /// Optional external event ID
        /// This is useful for external applications that need to link Nittei's events to their own data models
        /// Default is None
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub external_id: Option<String>,

        /// Optional location of the event
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub location: Option<String>,

        /// Optional status of the event
        /// Default is "Tentative"
        #[serde(default)]
        #[ts(optional, as = "Option<_>")]
        pub status: Option<CalendarEventStatus>,

        /// Optional flag to indicate if the event is an all day event
        /// Default is false
        #[serde(default)]
        #[ts(optional)]
        pub all_day: Option<bool>,

        /// Optional duration of the event in milliseconds
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

        /// Optional recurring event ID
        /// This is the ID of the recurring event that this event is part of
        /// Default is None
        #[serde(default)]
        #[ts(optional)]
        pub recurring_event_id: Option<ID>,

        /// Optional original start time of the event
        /// This is the original start time of the event before it was moved (only for recurring events)
        /// Default is None
        #[serde(default)]
        #[ts(optional, type = "Date")]
        pub original_start_time: Option<DateTime<Utc>>,

        /// Optional list of reminders
        #[serde(default)]
        #[ts(optional)]
        pub reminders: Option<Vec<CalendarEventReminder>>,

        /// Optional metadata (e.g. {"key": "value"})
        #[serde(default)]
        #[ts(optional)]
        pub metadata: Option<serde_json::Value>,

        /// Optional created date to use to replace the current one
        #[serde(default)]
        #[ts(optional, type = "Date")]
        pub created: Option<DateTime<Utc>>,

        /// Optional updated date to use to replace the current one
        #[serde(default)]
        #[ts(optional, type = "Date")]
        pub updated: Option<DateTime<Utc>>,
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
