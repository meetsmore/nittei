use nittei_domain::{Metadata, Service, ServiceResource, ServiceWithUsers, TimePlan, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// User service resource object
/// This is the configuration of a user for a service
#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ServiceResourceDTO {
    /// UUID of the user
    pub user_id: ID,
    /// UUID of the service
    pub service_id: ID,
    /// Availability of the user
    /// This allow to decide if the availability checks should be done
    /// on the user schedule or on the service schedule
    pub availability: TimePlan,
    /// Buffer after the booking time in minutes
    pub buffer_after: i64,
    /// Buffer before the booking time in minutes
    pub buffer_before: i64,
    /// Closest booking time in minutes
    pub closest_booking_time: i64,
    /// Optional furthest booking time in minutes
    pub furthest_booking_time: Option<i64>,
}

impl ServiceResourceDTO {
    pub fn new(resource: ServiceResource) -> Self {
        Self {
            user_id: resource.user_id,
            service_id: resource.service_id,
            availability: resource.availability,
            buffer_after: resource.buffer_after,
            buffer_before: resource.buffer_before,
            closest_booking_time: resource.closest_booking_time,
            furthest_booking_time: resource.furthest_booking_time,
        }
    }
}

/// Service object
#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ServiceDTO {
    /// UUID of the service
    pub id: ID,
    /// Metadata (e.g. {"key": "value"})
    #[ts(type = "Record<string, string | number | boolean>")]
    pub metadata: Metadata,
}

impl ServiceDTO {
    pub fn new(service: Service) -> Self {
        Self {
            id: service.id,
            metadata: service.metadata,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ServiceWithUsersDTO {
    pub id: ID,
    pub users: Vec<ServiceResourceDTO>,
    #[ts(type = "Record<string, string | number | boolean>")]
    pub metadata: Metadata,
}

impl ServiceWithUsersDTO {
    pub fn new(service: ServiceWithUsers) -> Self {
        Self {
            id: service.id,
            users: service
                .users
                .into_iter()
                .map(ServiceResourceDTO::new)
                .collect(),
            metadata: service.metadata,
        }
    }
}
