use crate::{shared::MetadataFindInput, APIResponse, BaseClient, TimePlan, Tz, ID};
use chrono::{DateTime, Utc};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::{BusyCalendar, Metadata, ServiceMultiPersonOptions};
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServiceClient {
    base: Arc<BaseClient>,
}

pub struct AddServiceUserInput {
    pub service_id: ID,
    pub user_id: ID,
    pub availability: Option<TimePlan>,
    pub buffer_after: Option<i64>,
    pub buffer_before: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

pub struct AddBusyCalendar {
    pub service_id: ID,
    pub user_id: ID,
    pub calendar: BusyCalendar,
}

pub struct RemoveBusyCalendar {
    pub service_id: ID,
    pub user_id: ID,
    pub calendar: BusyCalendar,
}

pub struct UpdateServiceUserInput {
    pub service_id: ID,
    pub user_id: ID,
    pub availability: Option<TimePlan>,
    pub buffer_after: Option<i64>,
    pub buffer_before: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

pub struct CreateBookingIntendInput {
    pub service_id: ID,
    pub host_user_ids: Option<Vec<ID>>,
    pub timestamp: DateTime<Utc>,
    pub duration: i64,
    pub interval: i64,
}

pub struct RemoveBookingIntendInput {
    pub service_id: ID,
    pub timestamp: DateTime<Utc>,
}

pub struct RemoveServiceUserInput {
    pub service_id: ID,
    pub user_id: ID,
}

#[derive(Debug, Clone)]
pub struct GetServiceBookingSlotsInput {
    pub service_id: ID,
    pub timezone: Option<Tz>,
    pub duration: i64,
    pub interval: i64,
    pub start_date: String,
    pub end_date: String,
    pub host_user_ids: Option<Vec<ID>>,
}

pub struct UpdateServiceInput {
    pub service_id: ID,
    pub metadata: Option<Metadata>,
    pub multi_person: Option<ServiceMultiPersonOptions>,
}

pub struct CreateServiceInput {
    pub metadata: Option<Metadata>,
    pub multi_person: Option<ServiceMultiPersonOptions>,
}

impl ServiceClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn get(&self, service_id: ID) -> APIResponse<get_service::APIResponse> {
        self.base
            .get(format!("service/{}", service_id), None, StatusCode::OK)
            .await
    }

    pub async fn bookingslots(
        &self,
        input: GetServiceBookingSlotsInput,
    ) -> APIResponse<get_service_bookingslots::APIResponse> {
        let mut query = vec![
            ("duration".to_string(), input.duration.to_string()),
            ("interval".to_string(), input.interval.to_string()),
            ("startDate".to_string(), input.start_date),
            ("endDate".to_string(), input.end_date),
        ];

        if let Some(timezone) = input.timezone {
            query.push(("ianaTz".to_string(), timezone.to_string()));
        }
        if let Some(host_user_ids) = input.host_user_ids {
            let host_user_ids = host_user_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            query.push(("hostUserIds".to_string(), host_user_ids));
        }

        self.base
            .get(
                format!("service/{}/booking", input.service_id),
                Some(query),
                StatusCode::OK,
            )
            .await
    }

    pub async fn create_booking_intend(
        &self,
        input: CreateBookingIntendInput,
    ) -> APIResponse<create_service_event_intend::APIResponse> {
        let body = create_service_event_intend::RequestBody {
            duration: input.duration,
            host_user_ids: input.host_user_ids,
            interval: input.interval,
            timestamp: input.timestamp,
        };
        self.base
            .post(
                body,
                format!("service/{}/booking-intend", input.service_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn remove_booking_intend(
        &self,
        input: RemoveBookingIntendInput,
    ) -> APIResponse<remove_service_event_intend::APIResponse> {
        self.base
            .delete(
                format!(
                    "service/{}/booking-intend?timestamp={}",
                    input.service_id, input.timestamp
                ),
                StatusCode::OK,
            )
            .await
    }

    pub async fn delete(&self, service_id: ID) -> APIResponse<delete_service::APIResponse> {
        self.base
            .delete(format!("service/{}", service_id), StatusCode::OK)
            .await
    }

    pub async fn create(
        &self,
        input: CreateServiceInput,
    ) -> APIResponse<create_service::APIResponse> {
        let body = create_service::RequestBody {
            metadata: input.metadata,
            multi_person: input.multi_person,
        };
        self.base
            .post(body, "service".into(), StatusCode::CREATED)
            .await
    }

    pub async fn update(
        &self,
        input: UpdateServiceInput,
    ) -> APIResponse<update_service::APIResponse> {
        let body = update_service::RequestBody {
            metadata: input.metadata,
            multi_person: input.multi_person,
        };
        self.base
            .put(
                body,
                format!("service/{}", input.service_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn get_by_meta(
        &self,
        input: MetadataFindInput,
    ) -> APIResponse<get_services_by_meta::APIResponse> {
        self.base
            .get(
                "service/meta".to_string(),
                Some(input.to_query()),
                StatusCode::OK,
            )
            .await
    }

    pub async fn remove_user(
        &self,
        input: RemoveServiceUserInput,
    ) -> APIResponse<remove_user_from_service::APIResponse> {
        self.base
            .delete(
                format!("service/{}/users/{}", input.service_id, input.user_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn update_user(
        &self,
        input: UpdateServiceUserInput,
    ) -> APIResponse<update_service_user::APIResponse> {
        let user_id = input.user_id.clone();
        let service_id = input.service_id.clone();
        let body = update_service_user::RequestBody {
            availability: input.availability,
            buffer_after: input.buffer_after,
            buffer_before: input.buffer_before,
            closest_booking_time: input.closest_booking_time,
            furthest_booking_time: input.furthest_booking_time,
        };

        self.base
            .put(
                body,
                format!("service/{}/users/{}", service_id, user_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn add_user(
        &self,
        input: AddServiceUserInput,
    ) -> APIResponse<add_user_to_service::APIResponse> {
        let service_id = input.service_id.clone();
        let body = add_user_to_service::RequestBody {
            user_id: input.user_id,
            availability: input.availability,
            buffer_after: input.buffer_after,
            buffer_before: input.buffer_before,
            closest_booking_time: input.closest_booking_time,
            furthest_booking_time: input.furthest_booking_time,
        };

        self.base
            .post(
                body,
                format!("service/{}/users", service_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn add_busy_calendar(
        &self,
        input: AddBusyCalendar,
    ) -> APIResponse<add_busy_calendar::APIResponse> {
        let body = add_busy_calendar::RequestBody {
            busy: input.calendar,
        };

        self.base
            .put(
                body,
                format!("service/{}/users/{}/busy", input.service_id, input.user_id),
                StatusCode::OK,
            )
            .await
    }

    pub async fn remove_busy_calendar(
        &self,
        input: RemoveBusyCalendar,
    ) -> APIResponse<remove_busy_calendar::APIResponse> {
        let body = remove_busy_calendar::RequestBody {
            busy: input.calendar,
        };

        self.base
            .delete_with_body(
                body,
                format!("service/{}/users/{}/busy", input.service_id, input.user_id),
                StatusCode::OK,
            )
            .await
    }
}
