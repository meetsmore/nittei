use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::Metadata;
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
    pub start_time: DateTime<FixedOffset>,
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

pub struct GetEventsInstancesInput {
    pub event_id: ID,
    pub start_time: DateTime<FixedOffset>,
    pub end_time: DateTime<FixedOffset>,
}

pub struct UpdateEventInput {
    pub event_id: ID,
    pub start_time: Option<DateTime<FixedOffset>>,
    pub duration: Option<i64>,
    pub busy: Option<bool>,
    pub reminders: Option<Vec<CalendarEventReminder>>,
    pub rrule_options: Option<RRuleOptions>,
    pub service_id: Option<ID>,
    pub exdates: Option<Vec<DateTime<FixedOffset>>>,
    pub metadata: Option<Metadata>,
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
