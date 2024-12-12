use chrono::{DateTime, Utc};
use nittei_domain::{
    CalendarEvent,
    CalendarEventReminder,
    CalendarEventStatus,
    EventInstance,
    RRuleOptions,
    ID,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Calendar event object
#[derive(Debug, Deserialize, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarEventDTO {
    /// UUID of the event
    pub id: ID,

    /// Optional title of the event
    pub title: Option<String>,

    /// Optional description of the event
    pub description: Option<String>,

    /// Optional location of the event
    pub location: Option<String>,

    /// Flag to indicate if the event is all day, default is false
    pub all_day: bool,

    /// Status of the event, default is tentative
    pub status: CalendarEventStatus,

    /// Optional parent event ID
    pub parent_id: Option<String>,

    /// Optional external ID
    pub external_id: Option<String>,

    /// Start time of the event (UTC)
    #[ts(type = "Date")]
    pub start_time: DateTime<Utc>,

    /// Start time of the event (UTC)
    #[ts(type = "Date")]
    pub end_time: DateTime<Utc>,

    /// Duration of the event in milliseconds
    #[ts(type = "number")]
    pub duration: i64,

    /// Busy flag
    pub busy: bool,

    /// Last updated timestamp
    #[ts(type = "number")]
    pub updated: i64,

    /// Created timestamp
    #[ts(type = "number")]
    pub created: i64,

    /// Recurrence rule
    #[ts(optional)]
    pub recurrence: Option<RRuleOptions>,

    /// List of exclusion dates for the recurrence rule
    #[ts(type = "Array<Date>")]
    pub exdates: Vec<DateTime<Utc>>,

    /// UUID of the calendar
    pub calendar_id: ID,

    /// UUID of the user
    pub user_id: ID,

    /// Optional group ID
    pub group_id: Option<ID>,

    /// List of reminders
    pub reminders: Vec<CalendarEventReminder>,

    /// Metadata (e.g. {"key": "value"})
    #[ts(optional)]
    pub metadata: Option<serde_json::Value>,
}

impl CalendarEventDTO {
    pub fn new(event: CalendarEvent) -> Self {
        Self {
            id: event.id,
            title: event.title,
            description: event.description,
            location: event.location,
            all_day: event.all_day,
            status: event.status,
            parent_id: event.parent_id,
            external_id: event.external_id,
            start_time: event.start_time,
            end_time: event.end_time,
            duration: event.duration,
            busy: event.busy,
            updated: event.updated,
            created: event.created,
            recurrence: event.recurrence,
            exdates: event.exdates,
            calendar_id: event.calendar_id,
            user_id: event.user_id,
            group_id: event.group_id,
            reminders: event.reminders,
            metadata: event.metadata,
        }
    }
}

/// Calendar event with instances
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventWithInstancesDTO {
    /// Calendar event
    pub event: CalendarEventDTO,
    /// List of event instances (e.g. recurring events)
    pub instances: Vec<EventInstance>,
}

impl EventWithInstancesDTO {
    pub fn new(event: CalendarEvent, instances: Vec<EventInstance>) -> Self {
        Self {
            event: CalendarEventDTO::new(event),
            instances,
        }
    }
}
