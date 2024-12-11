use nittei_domain::{event_group::EventGroup, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use super::dtos::EventGroupDTO;

/// Calendar event response object
#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventGroupResponse {
    /// Calendar event retrieved
    pub event_group: EventGroupDTO,
}

impl EventGroupResponse {
    pub fn new(event_group: EventGroup) -> Self {
        Self {
            event_group: EventGroupDTO::new(event_group),
        }
    }
}

pub mod create_event_group {

    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    /// Request body for creating an event
    #[derive(Serialize, Deserialize, Validate, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateEventGroupRequestBody")]
    pub struct RequestBody {
        /// UUID of the calendar where the event group will be created
        pub calendar_id: ID,

        /// Optional parent event ID
        /// This is useful for external applications that need to link Nittei's events to a wider data model (e.g. a project, an order, etc.)
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub parent_id: Option<String>,

        /// Optional external event ID
        /// This is useful for external applications that need to link Nittei's events to their own data models
        #[serde(default)]
        #[ts(optional)]
        #[validate(length(min = 1))]
        pub external_id: Option<String>,
    }

    pub type APIResponse = EventGroupResponse;
}

pub mod delete_event_group {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_group_id: ID,
    }

    pub type APIResponse = EventGroupResponse;
}

pub mod get_event_group {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_group_id: ID,
    }

    pub type APIResponse = EventGroupResponse;
}

pub mod get_event_group_by_external_id {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub external_id: String,
    }

    pub type APIResponse = EventGroupResponse;
}

pub mod update_event_group {

    use super::*;

    /// Request body for updating an event
    #[derive(Deserialize, Serialize, Validate, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "UpdateEventGroupRequestBody")]
    pub struct RequestBody {
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
    }

    #[derive(Deserialize)]
    pub struct PathParams {
        pub event_group_id: ID,
    }

    pub type APIResponse = EventGroupResponse;
}
