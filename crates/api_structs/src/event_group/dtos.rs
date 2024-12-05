use nittei_domain::{event_group::EventGroup, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Calendar event object
#[derive(Debug, Deserialize, Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventGroupDTO {
    /// UUID of the event
    pub id: ID,

    /// Optional parent event ID
    pub parent_id: Option<String>,

    /// Optional external ID
    pub external_id: Option<String>,

    /// UUID of the calendar
    pub calendar_id: ID,

    /// UUID of the user
    pub user_id: ID,
}

impl EventGroupDTO {
    /// Create a new EventGroupDTO from an EventGroup
    pub fn new(event_group: EventGroup) -> Self {
        Self {
            id: event_group.id,
            parent_id: event_group.parent_id,
            external_id: event_group.external_id,
            calendar_id: event_group.calendar_id,
            user_id: event_group.user_id,
        }
    }
}
