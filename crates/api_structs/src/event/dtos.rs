use chrono::{DateTime, Utc};
use nittei_domain::{
    CalendarEvent,
    CalendarEventReminder,
    CalendarEventStatus,
    EventInstance,
    ID,
    RRuleOptions,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// Calendar event object
#[derive(Debug, Deserialize, Serialize, Clone, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarEventDTO {
    /// UUID of the event
    pub id: ID,

    /// Optional title of the event
    #[ts(optional)]
    pub title: Option<String>,

    /// Optional description of the event
    #[ts(optional)]
    pub description: Option<String>,

    /// Optional type of the event
    /// e.g. "meeting", "reminder", "birthday"
    #[ts(optional)]
    pub event_type: Option<String>,

    /// Optional location of the event
    #[ts(optional)]
    pub location: Option<String>,

    /// Flag to indicate if the event is all day, default is false
    pub all_day: bool,

    /// Status of the event, default is tentative
    pub status: CalendarEventStatus,

    /// Optional parent event ID
    /// This is useful for external applications that need to link Nittei's events to a wider data model (e.g. a project, an order, etc.)
    /// Example: If the event is a meeting, the parent ID could be the project ID (ObjectId, UUID or any other string)
    #[ts(optional)]
    pub external_parent_id: Option<String>,

    /// Optional external ID
    /// This is useful for external applications that need to link Nittei's events to their own data models
    /// Example: If the event is a meeting, the external ID could be the meeting ID in the external system
    ///
    /// Note that nothing prevents multiple events from having the same external ID
    /// This can also be a way to link events together
    #[ts(optional)]
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

    /// Last updated timestamp (UTC)
    #[ts(type = "Date")]
    pub updated: DateTime<Utc>,

    /// Created tiemstamp (UTC)
    #[ts(type = "Date")]
    pub created: DateTime<Utc>,

    /// Recurrence rule
    #[ts(optional)]
    pub recurrence: Option<RRuleOptions>,

    /// Optional recurring until date
    /// This is the date until which the event will recur
    /// This is calculated by adding the duration to the until date
    #[ts(optional, type = "Date")]
    pub recurring_until: Option<DateTime<Utc>>,

    /// List of exclusion dates for the recurrence rule
    #[ts(type = "Array<Date>")]
    pub exdates: Vec<DateTime<Utc>>,

    /// Optional recurring event ID
    /// This is the ID of the recurring event that this event is part of
    /// Default is None
    #[ts(optional)]
    pub recurring_event_id: Option<ID>,

    /// Optional original start time of the event
    /// This is the original start time of the event before it was moved (only for recurring events)
    /// Default is None
    #[ts(type = "Date", optional)]
    pub original_start_time: Option<DateTime<Utc>>,

    /// UUID of the calendar
    pub calendar_id: ID,

    /// UUID of the user
    pub user_id: ID,

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
            event_type: event.event_type,
            location: event.location,
            all_day: event.all_day,
            status: event.status,
            external_parent_id: event.external_parent_id,
            external_id: event.external_id,
            start_time: event.start_time,
            end_time: event.end_time,
            duration: event.duration,
            busy: event.busy,
            updated: event.updated,
            created: event.created,
            recurrence: event.recurrence,
            recurring_until: event.recurring_until,
            exdates: event.exdates,
            recurring_event_id: event.recurring_event_id,
            original_start_time: event.original_start_time,
            calendar_id: event.calendar_id,
            user_id: event.user_id,
            reminders: event.reminders,
            metadata: event.metadata,
        }
    }
}

/// Calendar event with instances
#[derive(Serialize, Deserialize, Debug, Clone, TS, ToSchema)]
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
