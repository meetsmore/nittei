use std::sync::Arc;

use chrono::{DateTime, Utc};
use nittei_api_structs::*;
use nittei_domain::CalendarEventStatus;
use reqwest::StatusCode;
use serde::Serialize;

use crate::{
    shared::MetadataFindInput,
    APIResponse,
    BaseClient,
    CalendarEventReminder,
    RRuleOptions,
    ID,
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
    pub parent_id: Option<String>,
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
    pub parent_id: Option<String>,
    pub external_id: Option<String>,
    pub location: Option<String>,
    pub status: Option<CalendarEventStatus>,
    pub all_day: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub busy: Option<bool>,
    pub reminders: Option<Vec<CalendarEventReminder>>,
    pub rrule_options: Option<RRuleOptions>,
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
            .delete(format!("user/events/{}", event_id), StatusCode::OK)
            .await
    }

    pub async fn get(&self, event_id: ID) -> APIResponse<get_event::APIResponse> {
        self.base
            .get(format!("user/events/{}", event_id), None, StatusCode::OK)
            .await
    }

    pub async fn get_instances(
        &self,
        input: GetEventsInstancesInput,
    ) -> APIResponse<get_event_instances::APIResponse> {
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
        let body = create_event::RequestBody {
            parent_id: input.parent_id,
            external_id: input.external_id,
            title: input.title,
            description: input.description,
            location: input.location,
            status: input.status,
            all_day: input.all_day,
            calendar_id: input.calendar_id,
            start_time: input.start_time,
            duration: input.duration,
            busy: input.busy,
            recurrence: input.recurrence,
            reminders: input.reminders,
            service_id: input.service_id,
            metadata: input.metadata,
        };

        self.base
            .post(
                body,
                format!("user/{}/events", user_id),
                StatusCode::CREATED,
            )
            .await
    }

    pub async fn get_by_meta(
        &self,
        input: MetadataFindInput,
    ) -> APIResponse<get_events_by_meta::APIResponse> {
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
        let body = update_event::RequestBody {
            title: input.title,
            description: input.description,
            location: input.location,
            all_day: input.all_day,
            status: input.status,
            parent_id: input.parent_id,
            external_id: input.external_id,
            busy: input.busy,
            duration: input.duration,
            exdates: input.exdates,
            recurrence: input.rrule_options,
            reminders: input.reminders,
            service_id: input.service_id,
            start_time: input.start_time,
            metadata: input.metadata,
        };
        self.base
            .put(body, format!("user/events/{}", event_id), StatusCode::OK)
            .await
    }
}
