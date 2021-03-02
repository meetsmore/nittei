use nettu_scheduler_domain::Service;
use nettu_scheduler_domain::TimePlan;
use serde::{Deserialize, Serialize};

use crate::dtos::ServiceDTO;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResponse {
    service: ServiceDTO,
}

impl ServiceResponse {
    pub fn new(service: Service) -> Self {
        Self {
            service: ServiceDTO::new(&service),
        }
    }
}

pub mod add_user_to_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub user_id: String,
        pub availibility: Option<TimePlan>,
        pub busy: Option<Vec<String>>,
        pub buffer: Option<i64>,
        pub closest_booking_time: Option<i64>,
        pub furthest_booking_time: Option<i64>,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod create_service {
    use super::*;

    pub type APIResponse = ServiceResponse;
}

pub mod get_service_bookingslots {
    use nettu_scheduler_domain::{booking_slots::ServiceBookingSlot, User};

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub service_id: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub iana_tz: Option<String>,
        pub duration: i64,
        pub interval: i64,
        pub date: String,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ServiceBookingSlotDTO {
        pub start: i64,
        pub duration: i64,
        pub user_ids: Vec<String>,
    }

    impl ServiceBookingSlotDTO {
        pub fn new(slot: &ServiceBookingSlot) -> Self {
            Self {
                duration: slot.duration,
                start: slot.start,
                user_ids: slot
                    .user_ids
                    .iter()
                    .map(|u_id| User::create_external_id(u_id))
                    .collect(),
            }
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct APIResponse {
        pub booking_slots: Vec<ServiceBookingSlotDTO>,
    }

    impl APIResponse {
        pub fn new(booking_slots: Vec<ServiceBookingSlot>) -> Self {
            Self {
                booking_slots: booking_slots
                    .iter()
                    .map(|slot| ServiceBookingSlotDTO::new(slot))
                    .collect(),
            }
        }
    }
}

pub mod get_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: String,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod remove_user_from_service {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: String,
        pub user_id: String,
    }

    pub type APIResponse = ServiceResponse;
}

pub mod update_service_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub service_id: String,
        pub user_id: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        pub availibility: Option<TimePlan>,
        pub busy: Option<Vec<String>>,
        pub buffer: Option<i64>,
        pub closest_booking_time: Option<i64>,
        pub furthest_booking_time: Option<i64>,
    }

    pub type APIResponse = ServiceResponse;
}
