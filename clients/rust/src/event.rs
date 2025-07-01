use std::sync::Arc;

use chrono::{DateTime, Utc};
use nittei_api_structs::*;
use nittei_domain::CalendarEventStatus;
use reqwest::StatusCode;
use serde::Serialize;

use crate::{
    APIResponse,
    BaseClient,
    CalendarEventReminder,
    ID,
    RRuleOptions,
    shared::MetadataFindInput,
};

#[derive(Clone)]
pub struct CalendarEventClient {
    base: Arc<BaseClient>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventInput {
    pub user_id: ID,
    pub calendar_id: ID,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub external_parent_id: Option<String>,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub status: CalendarEventStatus,

    #[serde(default)]
    pub all_day: Option<bool>,

    pub start_time: DateTime<Utc>,

    pub duration: i64,

    #[serde(default)]
    pub busy: Option<bool>,
    #[serde(default)]
    pub recurrence: Option<RRuleOptions>,
    #[serde(default)]
    pub exdates: Option<Vec<DateTime<Utc>>>,
    #[serde(default)]
    pub recurring_event_id: Option<ID>,
    #[serde(default)]
    pub original_start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub reminders: Vec<CalendarEventReminder>,
    #[serde(default)]
    pub service_id: Option<ID>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

pub struct GetEventsInstancesInput {
    pub event_id: ID,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

pub struct UpdateEventInput {
    pub event_id: ID,
    pub title: Option<String>,
    pub description: Option<String>,
    pub event_type: Option<String>,
    pub external_parent_id: Option<String>,
    pub external_id: Option<String>,
    pub location: Option<String>,
    pub status: Option<CalendarEventStatus>,
    pub all_day: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub busy: Option<bool>,
    pub reminders: Option<Vec<CalendarEventReminder>>,
    pub recurrence: Option<RRuleOptions>,
    pub recurring_event_id: Option<ID>,
    pub original_start_time: Option<DateTime<Utc>>,
    pub service_id: Option<ID>,
    pub exdates: Option<Vec<DateTime<Utc>>>,
    pub metadata: Option<serde_json::Value>,
}

impl CalendarEventClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn delete(&self, event_id: ID) -> APIResponse<delete_event::APIResponse> {
        self.base
            .delete(format!("user/events/{event_id}"), StatusCode::OK)
            .await
    }

    pub async fn get(&self, event_id: ID) -> APIResponse<get_event::APIResponse> {
        self.base
            .get(format!("user/events/{event_id}"), None, StatusCode::OK)
            .await
    }

    pub async fn get_instances(
        &self,
        input: GetEventsInstancesInput,
    ) -> APIResponse<get_event_instances::GetEventInstancesAPIResponse> {
        self.base
            .get(
                format!("user/events/{}/instances", input.event_id),
                Some(vec![
                    ("startTime".to_string(), input.start_time.to_string()),
                    ("endTime".to_string(), input.end_time.to_string()),
                ]),
                StatusCode::OK,
            )
            .await
    }

    pub async fn create(&self, input: CreateEventInput) -> APIResponse<create_event::APIResponse> {
        let user_id = input.user_id.clone();
        let body = create_event::CreateEventRequestBody {
            external_parent_id: input.external_parent_id,
            external_id: input.external_id,
            title: input.title,
            description: input.description,
            event_type: input.event_type,
            location: input.location,
            status: input.status,
            all_day: input.all_day,
            calendar_id: input.calendar_id,
            start_time: input.start_time,
            duration: input.duration,
            busy: input.busy,
            recurrence: input.recurrence,
            exdates: input.exdates,
            recurring_event_id: input.recurring_event_id,
            original_start_time: input.original_start_time,
            reminders: input.reminders,
            service_id: input.service_id,
            metadata: input.metadata,
            created: None,
            updated: None,
        };

        self.base
            .post(body, format!("user/{user_id}/events"), StatusCode::CREATED)
            .await
    }

    pub async fn get_by_meta(
        &self,
        input: MetadataFindInput,
    ) -> APIResponse<get_events_by_meta::GetEventsByMetaAPIResponse> {
        self.base
            .get(
                "events/meta".to_string(),
                Some(input.to_query()),
                StatusCode::OK,
            )
            .await
    }

    pub async fn update(&self, input: UpdateEventInput) -> APIResponse<update_event::APIResponse> {
        let event_id = input.event_id.clone();
        let body = update_event::UpdateEventRequestBody {
            title: input.title,
            description: input.description,
            event_type: input.event_type,
            location: input.location,
            all_day: input.all_day,
            status: input.status,
            parent_id: input.external_parent_id,
            external_id: input.external_id,
            busy: input.busy,
            start_time: input.start_time,
            duration: input.duration,
            exdates: input.exdates,
            recurrence: input.recurrence,
            recurring_event_id: input.recurring_event_id,
            original_start_time: input.original_start_time,
            reminders: input.reminders,
            service_id: input.service_id,
            metadata: input.metadata,
            created: None,
            updated: None,
        };
        self.base
            .put(body, format!("user/events/{event_id}"), StatusCode::OK)
            .await
    }
}
