use nittei_domain::{Metadata, Service, ServiceResource, ServiceWithUsers, TimePlan, ID};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResourceDTO {
    pub user_id: ID,
    pub service_id: ID,
    pub availability: TimePlan,
    pub buffer_after: i64,
    pub buffer_before: i64,
    pub closest_booking_time: i64,
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDTO {
    pub id: ID,
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceWithUsersDTO {
    pub id: ID,
    pub users: Vec<ServiceResourceDTO>,
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
