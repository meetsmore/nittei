use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use axum_valid::Valid;
use nittei_api_structs::add_user_to_service::*;
use nittei_domain::{Account, ID, ServiceResource, TimePlan};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn add_user_to_service_controller(
    headers: HeaderMap,
    mut body: Valid<Json<RequestBody>>,
    mut path: Path<PathParams>,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

    let usecase = AddUserToServiceUseCase {
        account,
        service_id: std::mem::take(&mut path.service_id),
        user_id: std::mem::take(&mut body.user_id),
        availability: std::mem::take(&mut body.availability),
        buffer_before: body.buffer_before,
        buffer_after: body.buffer_after,
        closest_booking_time: body.closest_booking_time,
        furthest_booking_time: body.furthest_booking_time,
    };

    execute(usecase, &ctx)
        .await
        .map(|res| Json(APIResponse::new(res.user)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
struct AddUserToServiceUseCase {
    pub account: Account,
    pub service_id: ID,
    pub user_id: ID,
    pub availability: Option<TimePlan>,
    pub buffer_before: Option<i64>,
    pub buffer_after: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

#[derive(Debug)]
struct UseCaseRes {
    pub user: ServiceResource,
}

#[derive(Debug)]
enum UseCaseError {
    InternalError,
    ServiceNotFound,
    UserNotFound,
    UserAlreadyInService,
    InvalidValue(UpdateServiceResourceError),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::ServiceNotFound => Self::NotFound("The requested service was not found".into()),
            UseCaseError::UserNotFound => Self::NotFound("The specified user was not found".into()),
            UseCaseError::UserAlreadyInService => Self::Conflict("The specified user is already registered on the service, can not add the user more than once.".into()),
            UseCaseError::InvalidValue(e) => e.to_nittei_error(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for AddUserToServiceUseCase {
    type Response = UseCaseRes;

    type Error = UseCaseError;

    const NAME: &'static str = "AddUserToService";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        if ctx
            .repos
            .users
            .find_by_account_id(&self.user_id, &self.account.id)
            .await
            .map_err(|_| UseCaseError::InternalError)?
            .is_none()
        {
            return Err(UseCaseError::UserNotFound);
        }

        let service = match ctx.repos.services.find(&self.service_id).await {
            Ok(Some(service)) if service.account_id == self.account.id => service,
            Ok(_) => return Err(UseCaseError::ServiceNotFound),
            Err(_) => return Err(UseCaseError::InternalError),
        };

        let mut user_resource =
            ServiceResource::new(self.user_id.clone(), service.id.clone(), TimePlan::Empty);

        update_resource_values(
            &mut user_resource,
            &ServiceResourceUpdate {
                availability: self.availability.clone(),
                buffer_after: self.buffer_after,
                buffer_before: self.buffer_before,
                closest_booking_time: self.closest_booking_time,
                furthest_booking_time: self.furthest_booking_time,
            },
            ctx,
        )
        .await
        .map_err(UseCaseError::InvalidValue)?;

        ctx.repos
            .service_users
            .insert(&user_resource)
            .await
            .map(|_| UseCaseRes {
                user: user_resource,
            })
            .map_err(|_| UseCaseError::UserAlreadyInService)
    }
}

#[derive(Debug)]
pub struct ServiceResourceUpdate {
    pub availability: Option<TimePlan>,
    pub buffer_after: Option<i64>,
    pub buffer_before: Option<i64>,
    pub closest_booking_time: Option<i64>,
    pub furthest_booking_time: Option<i64>,
}

#[derive(Debug)]
pub enum UpdateServiceResourceError {
    InternalError,
    InvalidBuffer,
    CalendarNotOwnedByUser(String),
    ScheduleNotOwnedByUser(String),
    InvalidBookingTimespan(String),
}

impl UpdateServiceResourceError {
    pub fn to_nittei_error(&self) -> NitteiError {
        match self {
            Self::InternalError => NitteiError::InternalError,
            Self::InvalidBuffer => {
                NitteiError::BadClientData("The provided buffer was invalid, it should be between 0 and 12 hours specified in minutes.".into())
            }
            Self::CalendarNotOwnedByUser(calendar_id) => NitteiError::NotFound(format!("The calendar: {}, was not found among the calendars for the specified user", calendar_id)),
            Self::ScheduleNotOwnedByUser(schedule_id) => {
                NitteiError::NotFound(format!(
                    "The schedule with id: {}, was not found among the schedules for the specified user",
                    schedule_id
                ))
            }
            Self::InvalidBookingTimespan(e) => {
                NitteiError::BadClientData(e.to_string())
            }
        }
    }
}

pub async fn update_resource_values(
    user_resource: &mut ServiceResource,
    update: &ServiceResourceUpdate,
    ctx: &NitteiContext,
) -> Result<(), UpdateServiceResourceError> {
    if let Some(availability) = &update.availability {
        match availability {
            TimePlan::Calendar(id) => {
                match ctx.repos.calendars.find(id).await {
                    Ok(Some(cal)) if cal.user_id == user_resource.user_id => {}
                    Ok(_) => {
                        return Err(UpdateServiceResourceError::CalendarNotOwnedByUser(
                            id.to_string(),
                        ));
                    }
                    Err(_) => {
                        return Err(UpdateServiceResourceError::InternalError);
                    }
                };
            }
            TimePlan::Schedule(id) => match ctx.repos.schedules.find(id).await {
                Ok(Some(schedule)) if schedule.user_id == user_resource.user_id => {}
                Ok(_) => {
                    return Err(UpdateServiceResourceError::ScheduleNotOwnedByUser(
                        id.to_string(),
                    ));
                }
                Err(_) => {
                    return Err(UpdateServiceResourceError::InternalError);
                }
            },
            _ => (),
        };
        user_resource.set_availability(availability.clone());
    }

    if let Some(buffer) = update.buffer_after {
        if !user_resource.set_buffer_after(buffer) {
            return Err(UpdateServiceResourceError::InvalidBuffer);
        }
    }
    if let Some(buffer) = update.buffer_before {
        if !user_resource.set_buffer_before(buffer) {
            return Err(UpdateServiceResourceError::InvalidBuffer);
        }
    }

    if let Some(closest_booking_time) = update.closest_booking_time {
        if closest_booking_time < 0 {
            return Err(UpdateServiceResourceError::InvalidBookingTimespan(
                "Closest booking time cannot be negative.".into(),
            ));
        }
        user_resource.closest_booking_time = closest_booking_time;
    }

    if let Some(furthest_booking_time) = &update.furthest_booking_time {
        if *furthest_booking_time < 0 {
            return Err(UpdateServiceResourceError::InvalidBookingTimespan(
                "Furthest booking time cannot be negative.".into(),
            ));
        }
    }
    user_resource.furthest_booking_time = update.furthest_booking_time;

    Ok(())
}
