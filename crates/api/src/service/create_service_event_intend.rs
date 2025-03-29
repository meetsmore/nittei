use axum::{Extension, Json, extract::Path, http::HeaderMap};
use chrono::{DateTime, Duration, TimeDelta, Utc};
use get_service_bookingslots::GetServiceBookingSlotsUseCase;
use nittei_api_structs::create_service_event_intend::*;
use nittei_domain::{
    ID,
    ServiceMultiPersonOptions,
    User,
    format_date,
    scheduling::{
        RoundRobinAlgorithm,
        RoundRobinAvailabilityAssignment,
        RoundRobinEqualDistributionAssignment,
    },
};
use nittei_infra::NitteiContext;
use tracing::warn;

use super::get_service_bookingslots;
use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn create_service_event_intend_controller(
    headers: HeaderMap,
    mut path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
    body: Json<RequestBody>,
) -> Result<Json<APIResponse>, NitteiError> {
    protect_admin_route(&headers, &ctx).await?;

    let mut body = body.0;
    let usecase = CreateServiceEventIntendUseCase {
        service_id: std::mem::take(&mut path.service_id),
        host_user_ids: body.host_user_ids.take(),
        duration: body.duration,
        timestamp: body.timestamp,
        interval: body.interval,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| {
            Json(APIResponse::new(
                res.selected_hosts,
                res.create_event_for_hosts,
            ))
        })
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct CreateServiceEventIntendUseCase {
    pub service_id: ID,
    pub host_user_ids: Option<Vec<ID>>,
    pub timestamp: DateTime<Utc>,
    pub duration: i64,
    pub interval: i64,
}

#[derive(Debug)]
struct UseCaseRes {
    pub selected_hosts: Vec<User>,
    pub create_event_for_hosts: bool,
}

#[derive(Debug)]
enum UseCaseError {
    UserNotAvailable,
    StorageError,
    BookingSlotsQuery(get_service_bookingslots::UseCaseError),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::UserNotAvailable => {
                Self::BadClientData("The user is not available at the given time".into())
            }
            UseCaseError::StorageError => Self::InternalError,
            UseCaseError::BookingSlotsQuery(e) => e.into(),
        }
    }
}

#[async_trait::async_trait]
impl UseCase for CreateServiceEventIntendUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateServiceEventIntend";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let start = self.timestamp;
        let start_date = format_date(&start);
        let day_after = start + Duration::days(1);
        let end_date = format_date(&day_after);

        let get_bookingslots_usecase = GetServiceBookingSlotsUseCase {
            duration: self.duration,
            service_id: self.service_id.clone(),
            end_date,
            start_date,
            timezone: Some(chrono_tz::UTC),
            interval: self.interval,
            host_user_ids: self.host_user_ids.clone(),
        };
        let res = execute(get_bookingslots_usecase, ctx)
            .await
            .map_err(UseCaseError::BookingSlotsQuery)?;
        let service = res.service;
        let booking_slots_dates = res.booking_slots.dates;

        let mut create_event_for_hosts = true;
        let selected_host_user_ids = if let Some(host_user_ids) = &self.host_user_ids {
            let mut found_slot = false;
            for date in booking_slots_dates {
                for slot in date.slots {
                    if slot.start == self.timestamp {
                        // Check that all host users are available
                        for host_user_id in host_user_ids {
                            if !slot.user_ids.contains(host_user_id) {
                                return Err(UseCaseError::UserNotAvailable);
                            }
                        }
                        found_slot = true;
                        break;
                    }
                    if slot.start > self.timestamp {
                        break;
                    }
                }
                if found_slot {
                    break;
                }
            }
            if !found_slot {
                return Err(UseCaseError::UserNotAvailable);
            }
            host_user_ids.clone()
        } else {
            let mut hosts_at_slot = Vec::new();
            for date in booking_slots_dates {
                for slot in date.slots {
                    if slot.start == self.timestamp {
                        hosts_at_slot.clone_from(&slot.user_ids);
                        break;
                    }
                    if slot.start > self.timestamp {
                        return Err(UseCaseError::UserNotAvailable);
                    }
                }
                if !hosts_at_slot.is_empty() {
                    break;
                }
            }
            let hosts_at_slot = service
                .users
                .iter()
                .filter(|member| hosts_at_slot.contains(&member.user_id))
                .collect::<Vec<_>>();

            if hosts_at_slot.is_empty() {
                return Err(UseCaseError::UserNotAvailable);
            } else {
                let user_ids_at_slot = hosts_at_slot
                    .iter()
                    .map(|h| h.user_id.clone())
                    .collect::<Vec<_>>();
                // Do round robin to get host
                match &service.multi_person {
                    ServiceMultiPersonOptions::RoundRobinAlgorithm(round_robin) => {
                        match round_robin {
                            RoundRobinAlgorithm::Availability => {
                                if hosts_at_slot.len() == 1 {
                                    vec![hosts_at_slot[0].user_id.clone()]
                                } else {
                                    let events = ctx
                                        .repos
                                        .events
                                        .find_most_recently_created_service_events(
                                            &service.id,
                                            &user_ids_at_slot,
                                        )
                                        .await
                                        .map_err(|_| UseCaseError::StorageError)?;

                                    let query = RoundRobinAvailabilityAssignment {
                                        members: events
                                            .into_iter()
                                            .map(|e| (e.user_id, e.created))
                                            .collect::<Vec<(ID, Option<DateTime<Utc>>)>>(),
                                    };
                                    let selected_user_id = query.assign().ok_or_else(|| {
                                        warn!("At least one host can be picked when there are at least one host available");
                                        UseCaseError::UserNotAvailable
                                    })?;
                                    vec![selected_user_id]
                                }
                            }
                            RoundRobinAlgorithm::EqualDistribution => {
                                if hosts_at_slot.len() == 1 {
                                    vec![hosts_at_slot[0].user_id.clone()]
                                } else {
                                    let now = Utc::now();
                                    let timestamp_in_two_months =
                                        now + TimeDelta::milliseconds(1000 * 60 * 60 * 24 * 61);

                                    let service_events = ctx
                                        .repos
                                        .events
                                        .find_by_service(
                                            &service.id,
                                            &user_ids_at_slot,
                                            now,
                                            timestamp_in_two_months,
                                        )
                                        .await
                                        .map_err(|_| UseCaseError::StorageError)?;

                                    let query = RoundRobinEqualDistributionAssignment {
                                        events: service_events,
                                        user_ids: user_ids_at_slot,
                                    };
                                    let selected_user_id = query.assign().ok_or_else(|| {
                                        warn!("At least one host can be picked when there are at least one host available");
                                        UseCaseError::UserNotAvailable
                                    })?;
                                    vec![selected_user_id]
                                }
                            }
                        }
                    }
                    ServiceMultiPersonOptions::Collective => {
                        let all_hosts_user_ids: Vec<_> = service
                            .users
                            .iter()
                            .map(|resource| resource.user_id.clone())
                            .collect();

                        // Check that all the hosts are available
                        if user_ids_at_slot.len() < all_hosts_user_ids.len() {
                            return Err(UseCaseError::UserNotAvailable);
                        }

                        all_hosts_user_ids
                    }
                    ServiceMultiPersonOptions::Group(max_count) => {
                        let all_hosts_user_ids: Vec<_> = service
                            .users
                            .iter()
                            .map(|resource| resource.user_id.clone())
                            .collect();

                        // Check that all the hosts are available
                        if user_ids_at_slot.len() < all_hosts_user_ids.len() {
                            return Err(UseCaseError::UserNotAvailable);
                        }

                        let reservations = ctx
                            .repos
                            .reservations
                            .count(&service.id, self.timestamp)
                            .await
                            .map_err(|_| UseCaseError::StorageError)?;
                        if reservations + 1 < *max_count {
                            // Client do not need to create service event yet
                            create_event_for_hosts = false;
                        }

                        ctx.repos
                            .reservations
                            .increment(&service.id, self.timestamp)
                            .await
                            .map_err(|_| UseCaseError::StorageError)?;

                        all_hosts_user_ids
                    }
                }
            }
        };

        let selected_hosts = ctx
            .repos
            .users
            .find_many(&selected_host_user_ids)
            .await
            .map_err(|_| UseCaseError::StorageError)?;

        Ok(UseCaseRes {
            selected_hosts,
            create_event_for_hosts,
        })
    }
}
