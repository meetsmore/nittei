use nittei_domain::{
    BusyCalendarProvider,
    ID,
    Service,
    ServiceResource,
    ServiceWithUsers,
    TimePlan,
    Tz,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dtos::{ServiceDTO, ServiceResourceDTO, ServiceWithUsersDTO};

#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ServiceResponse {
    pub service: ServiceDTO,
}

impl ServiceResponse {
    pub fn new(service: Service) -> Self {
        Self {
            service: ServiceDTO::new(service),
        }
    }
}

#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ServiceWithUsersResponse {
    pub service: ServiceWithUsersDTO,
}

impl ServiceWithUsersResponse {
    pub fn new(service: ServiceWithUsers) -> Self {
        Self {
            service: ServiceWithUsersDTO::new(service),
        }
    }
}

#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ServiceResourceResponse {
    pub user: ServiceResourceDTO,
}

impl ServiceResourceResponse {
    pub fn new(user: ServiceResource) -> Self {
        Self {
            user: ServiceResourceDTO::new(user),
        }
    }
}

pub mod add_user_to_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "AddUserToServiceRequestBody", optional_fields)]
    pub struct RequestBody {
        pub user_id: ID,
        pub availability: Option<TimePlan>,
        #[serde(default)]
        #[ts(type = "number")]
        pub buffer_after: Option<i64>,
        #[serde(default)]
        #[ts(type = "number")]
        pub buffer_before: Option<i64>,
        #[ts(type = "number")]
        pub closest_booking_time: Option<i64>,
        #[ts(type = "number")]
        pub furthest_booking_time: Option<i64>,
    }

    pub type APIResponse = ServiceResourceDTO;
}

pub mod add_busy_calendar {
    use super::*;

    #[derive(Deserialize, TS)]
    #[ts(export, rename_all = "camelCase", rename = "AddBusyCalendarPathParams")]
    pub struct PathParams {
        pub service_id: ID,
        pub user_id: ID,
    }

    #[derive(Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "AddBusyCalendarRequestBody")]
    pub struct RequestBody {
        pub busy: BusyCalendarProvider,
    }

    pub type APIResponse = String;
}

pub mod remove_busy_calendar {
    use super::*;

    #[derive(Deserialize, TS)]
    #[ts(
        export,
        rename_all = "camelCase",
        rename = "RemoveBusyCalendarPathParams"
    )]
    pub struct PathParams {
        pub service_id: ID,
        pub user_id: ID,
    }

    #[derive(Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "RemoveBusyCalendarRequestBody")]
    pub struct RequestBody {
        pub busy: BusyCalendarProvider,
    }

    pub type APIResponse = String;
}

pub mod remove_service_event_intend {
    use chrono::{DateTime, Utc};

    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub timestamp: DateTime<Utc>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        message: String,
    }

    impl Default for APIResponse {
        fn default() -> Self {
            Self {
                message: "Deleted Booking Intend".into(),
            }
        }
    }
}

pub mod create_service_event_intend {
    use chrono::{DateTime, Utc};
    use nittei_domain::User;

    use super::*;
    use crate::dtos::UserDTO;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateServiceEventIntendRequestBody")]
    pub struct RequestBody {
        #[serde(default)]
        pub host_user_ids: Option<Vec<ID>>,
        pub timestamp: DateTime<Utc>,
        pub duration: i64,
        pub interval: i64,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub selected_hosts: Vec<UserDTO>,
        pub create_event_for_hosts: bool,
    }

    impl APIResponse {
        pub fn new(selected_hosts: Vec<User>, create_event_for_hosts: bool) -> Self {
            Self {
                selected_hosts: selected_hosts.into_iter().map(UserDTO::new).collect(),
                create_event_for_hosts,
            }
        }
    }
}

pub mod create_service {
    use nittei_domain::ServiceMultiPersonOptions;

    use super::*;

    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateServiceRequestBody")]
    pub struct RequestBody {
        #[serde(default)]
        #[ts(optional)]
        pub metadata: Option<serde_json::Value>,
        #[serde(default)]
        pub multi_person: Option<ServiceMultiPersonOptions>,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod update_service {
    use nittei_domain::ServiceMultiPersonOptions;

    use super::*;

    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "UpdateServiceRequestBody")]
    pub struct RequestBody {
        #[serde(default)]
        #[ts(optional)]
        pub metadata: Option<serde_json::Value>,
        #[serde(default)]
        pub multi_person: Option<ServiceMultiPersonOptions>,
    }

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod get_service_bookingslots {
    use chrono::{DateTime, Utc};
    use nittei_domain::booking_slots::{
        ServiceBookingSlot,
        ServiceBookingSlots,
        ServiceBookingSlotsDate,
    };

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    #[derive(Debug, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetServiceBookingSlotsQueryParams", optional_fields)]
    pub struct QueryParams {
        #[ts(type = "string")]
        pub timezone: Option<Tz>,
        #[ts(type = "number")]
        pub duration: i64,
        #[ts(type = "number")]
        pub interval: i64,
        pub start_date: String,
        pub end_date: String,
        #[serde(default)]
        pub host_user_ids: Option<String>,
    }

    #[derive(Deserialize, Serialize, Debug, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct ServiceBookingSlotDTO {
        pub start: DateTime<Utc>,
        pub duration: i64,
        pub user_ids: Vec<ID>,
    }

    impl ServiceBookingSlotDTO {
        pub fn new(slot: ServiceBookingSlot) -> Self {
            Self {
                duration: slot.duration,
                start: slot.start,
                user_ids: slot.user_ids,
            }
        }
    }

    #[derive(Deserialize, Serialize, Debug, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct ServiceBookingSlotsDateDTO {
        pub date: String,
        pub slots: Vec<ServiceBookingSlotDTO>,
    }

    impl ServiceBookingSlotsDateDTO {
        pub fn new(date_slots: ServiceBookingSlotsDate) -> Self {
            Self {
                date: date_slots.date,
                slots: date_slots
                    .slots
                    .into_iter()
                    .map(ServiceBookingSlotDTO::new)
                    .collect(),
            }
        }
    }

    #[derive(Deserialize, Serialize, Debug, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetServiceBookingSlotsAPIResponse")]
    pub struct APIResponse {
        pub dates: Vec<ServiceBookingSlotsDateDTO>,
    }

    impl APIResponse {
        pub fn new(booking_slots: ServiceBookingSlots) -> Self {
            Self {
                dates: booking_slots
                    .dates
                    .into_iter()
                    .map(ServiceBookingSlotsDateDTO::new)
                    .collect(),
            }
        }
    }
}

pub mod get_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    pub type APIResponse = ServiceWithUsersDTO;
}

pub mod get_services_by_meta {
    use super::*;
    use crate::dtos::ServiceDTO;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub key: String,
        pub value: String,
        #[serde(default)]
        pub skip: Option<usize>,
        pub limit: Option<usize>,
    }

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetServicesByMetaAPIResponse")]
    pub struct APIResponse {
        pub services: Vec<ServiceDTO>,
    }

    impl APIResponse {
        pub fn new(services: Vec<Service>) -> Self {
            Self {
                services: services.into_iter().map(ServiceDTO::new).collect(),
            }
        }
    }
}

pub mod delete_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod remove_user_from_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
        pub user_id: ID,
    }

    pub type APIResponse = String;
}

pub mod update_service_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: ID,
        pub user_id: ID,
    }

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "UpdateServiceUserRequestBody")]
    pub struct RequestBody {
        pub availability: Option<TimePlan>,
        pub buffer_after: Option<i64>,
        pub buffer_before: Option<i64>,
        pub closest_booking_time: Option<i64>,
        pub furthest_booking_time: Option<i64>,
    }

    pub type APIResponse = ServiceResourceDTO;
}
