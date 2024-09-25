use chrono::{DateTime, Utc};
use nittei_domain::{
    CalendarEvent,
    CalendarEventReminder,
    EventInstance,
    Metadata,
    RRuleOptions,
    ID,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Deserialize, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarEventDTO {
    pub id: ID,
    #[ts(type = "Date")]
    pub start_time: DateTime<Utc>,
    #[ts(type = "number")]
    pub duration: i64,
    pub busy: bool,
    #[ts(type = "number")]
    pub updated: i64,
    #[ts(type = "number")]
    pub created: i64,
    #[ts(optional)]
    pub recurrence: Option<RRuleOptions>,
    #[ts(type = "Array<Date>")]
    pub exdates: Vec<DateTime<Utc>>,
    pub calendar_id: ID,
    pub user_id: ID,
    pub reminders: Vec<CalendarEventReminder>,
    #[ts(type = "Record<string, string>")]
    pub metadata: Metadata,
}

impl CalendarEventDTO {
    pub fn new(event: CalendarEvent) -> Self {
        Self {
            id: event.id.clone(),
            start_time: event.start_time,
            duration: event.duration,
            busy: event.busy,
            updated: event.updated,
            created: event.created,
            recurrence: event.recurrence,
            exdates: event.exdates,
            calendar_id: event.calendar_id.clone(),
            user_id: event.user_id.clone(),
            reminders: event.reminders,
            metadata: event.metadata,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventWithInstancesDTO {
    pub event: CalendarEventDTO,
    pub instances: Vec<EventInstance>,
}
