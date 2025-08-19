use axum::{Extension, Json, extract::Path};
use axum_valid::Valid;
use chrono::{DateTime, TimeDelta, Utc};
use event::subscribers::SyncRemindersOnEventUpdated;
use nittei_api_structs::update_event::*;
use nittei_domain::{
    Account,
    CalendarEvent,
    CalendarEventReminder,
    CalendarEventStatus,
    ID,
    RRuleOptions,
    User,
};
use nittei_infra::NitteiContext;
use nittei_utils::config::APP_CONFIG;

use crate::{
    error::NitteiError,
    event::{self, subscribers::UpdateSyncedEventsOnEventUpdated},
    shared::{
        auth::{Permission, Policy, account_can_modify_user},
        usecase::{PermissionBoundary, Subscriber, UseCase, execute, execute_with_policy},
    },
};

#[utoipa::path(
    put,
    tag = "Event",
    path = "/api/v1/user/events/{event_id}",
    summary = "Update an event (admin only)",
    params(
        ("event_id" = ID, Path, description = "The id of the event to update"),
    ),
    security(
        ("api_key" = [])
    ),
    request_body(
        content = UpdateEventRequestBody,
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn update_event_admin_controller(
    Extension(account): Extension<Account>,
    Extension(event): Extension<CalendarEvent>,
    Extension(ctx): Extension<NitteiContext>,
    body: Valid<Json<UpdateEventRequestBody>>,
) -> Result<Json<APIResponse>, NitteiError> {
    let user = account_can_modify_user(&account, &event.user_id, &ctx).await?;

    let mut body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: event.id.clone(),
        title: body.title.take(),
        description: body.description.take(),
        event_type: body.event_type.take(),
        external_parent_id: body.external_parent_id.take(),
        external_id: body.external_id.take(),
        location: body.location.take(),
        status: body.status.take(),
        all_day: body.all_day,
        duration: body.duration,
        start_time: body.start_time,
        reminders: body.reminders.take(),
        busy: body.busy,
        service_id: body.service_id.take(),
        recurrence: body.recurrence.take(),
        exdates: body.exdates.take(),
        recurring_event_id: body.recurring_event_id.take(),
        original_start_time: body.original_start_time,
        metadata: body.metadata.take(),
        created: body.created,
        updated: body.updated,

        // Prefetched event by the route guard
        prefetched_calendar_event: Some(event),
    };

    execute(usecase, &ctx)
        .await
        .map(|event| Json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

#[utoipa::path(
    put,
    tag = "Event",
    path = "/api/v1/events/{event_id}",
    summary = "Update an event (user only)",
    params(
        ("event_id" = ID, Path, description = "The id of the event to update"),
    ),
    request_body(
        content = UpdateEventRequestBody,
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn update_event_controller(
    path_params: Path<PathParams>,
    Extension((user, policy)): Extension<(User, Policy)>,
    Extension(ctx): Extension<NitteiContext>,
    body: Valid<Json<UpdateEventRequestBody>>,
) -> Result<Json<APIResponse>, NitteiError> {
    let mut body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: path_params.event_id.clone(),
        title: body.title.take(),
        description: body.description.take(),
        event_type: body.event_type.take(),
        external_parent_id: body.external_parent_id.take(),
        external_id: body.external_id.take(),
        location: body.location.take(),
        status: body.status.take(),
        all_day: body.all_day,
        duration: body.duration,
        start_time: body.start_time,
        reminders: body.reminders.take(),
        busy: body.busy,
        service_id: body.service_id.take(),
        recurrence: body.recurrence.take(),
        exdates: body.exdates.take(),
        recurring_event_id: body.recurring_event_id.take(),
        original_start_time: body.original_start_time,
        metadata: body.metadata.take(),
        created: body.created,
        updated: body.updated,
        prefetched_calendar_event: None,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| Json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

/// Use case for updating an event
#[derive(Debug, Default)]
pub struct UpdateEventUseCase {
    pub user: User,
    pub event_id: ID,

    pub title: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub event_type: Option<Option<String>>,
    pub external_parent_id: Option<Option<String>>,
    pub external_id: Option<Option<String>>,
    pub location: Option<Option<String>>,
    pub status: Option<CalendarEventStatus>,
    pub all_day: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub busy: Option<bool>,
    pub duration: Option<i64>,
    pub reminders: Option<Vec<CalendarEventReminder>>,
    pub service_id: Option<Option<ID>>,
    pub recurrence: Option<Option<RRuleOptions>>,
    pub exdates: Option<Vec<DateTime<Utc>>>,
    pub recurring_event_id: Option<Option<ID>>,
    pub original_start_time: Option<Option<DateTime<Utc>>>,
    pub metadata: Option<Option<serde_json::Value>>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,

    /// Event that has been potentially prefetched by the route guard
    /// Only happens in the admin controller
    pub prefetched_calendar_event: Option<CalendarEvent>,
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(String, ID),
    InvalidReminder,
    StorageError,
    InvalidRecurrenceRule,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(entity, event_id) => {
                Self::NotFound(format!("The {entity} with id: {event_id}, was not found."))
            }
            UseCaseError::InvalidRecurrenceRule => {
                Self::BadClientData("Invalid recurrence rule specified for the event".into())
            }
            UseCaseError::InvalidReminder => {
                Self::BadClientData("Invalid reminder specified for the event".into())
            }
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for UpdateEventUseCase {
    type Response = CalendarEvent;

    type Error = UseCaseError;

    const NAME: &'static str = "UpdateEvent";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let UpdateEventUseCase {
            user,
            event_id,
            title,
            description,
            event_type,
            external_parent_id,
            external_id,
            location,
            status,
            all_day,
            start_time,
            busy,
            duration,
            recurrence,
            exdates,
            recurring_event_id,
            original_start_time,
            reminders,
            service_id,
            metadata,
            created,
            updated,
            prefetched_calendar_event,
        } = self;

        println!("title: {:?}", title);

        let e = match &prefetched_calendar_event {
            Some(event) => Some(event.clone()),
            None => ctx.repos.events.find(event_id).await.map_err(|e| {
                tracing::error!("[update_event] Error finding event: {:?}", e);
                UseCaseError::StorageError
            })?,
        };

        let mut e = match e {
            Some(event) if event.user_id == user.id => event,
            Some(_) => {
                return Err(UseCaseError::NotFound(
                    "Calendar Event".into(),
                    event_id.clone(),
                ));
            }
            None => {
                return Err(UseCaseError::NotFound(
                    "Calendar Event".into(),
                    event_id.clone(),
                ));
            }
        };

        if let Some(service_id_value) = service_id {
            if service_id_value.is_some() {
                e.service_id = service_id_value.take();
            } else {
                // Set to NULL
                e.service_id = None;
            }
        }

        if let Some(exdates_value) = exdates {
            e.exdates = exdates_value.clone();
        }

        if let Some(metadata_value) = metadata {
            if metadata_value.is_some() {
                e.metadata = metadata_value.take();
            } else {
                // Set to NULL
                e.metadata = None;
            }
        }

        if let Some(reminders) = &reminders {
            for reminder in reminders {
                if !reminder.is_valid() {
                    tracing::warn!("[update_event] Invalid reminder");
                    return Err(UseCaseError::InvalidReminder);
                }
            }
            e.reminders.clone_from(reminders);
        }

        let mut start_or_duration_change = false;

        if let Some(start_time_value) = start_time {
            // Only change the exdates if the start time has actually changed
            if e.start_time != *start_time_value {
                e.exdates = Vec::new();
            }
            e.start_time = *start_time_value;
            start_or_duration_change = true;
        }
        if let Some(duration_value) = duration {
            e.duration = *duration_value;
            start_or_duration_change = true;
        }

        if start_or_duration_change {
            e.end_time = e.start_time + TimeDelta::milliseconds(e.duration);
        }

        if let Some(busy_value) = busy {
            e.busy = *busy_value;
        }

        // Handle the new recurrence
        let valid_recurrence = if let Some(recurrence_value) = &recurrence {
            if let Some(rrule_opts) = recurrence_value {
                // ? should exdates be deleted when rrules are updated
                e.set_recurrence(rrule_opts.clone()).map_err(|e| {
                    tracing::error!("[update_event] Failed to set recurrence {:?}", e);
                    UseCaseError::InvalidRecurrenceRule
                })?
            } else {
                // Set to NULL
                e.recurrence = None;
                true
            }
        // Otherwise, we we don't have a new recurrence, but we have an existing one
        // And the start time or duration has changed, we need to update the recurrence
        } else if start_or_duration_change {
            if e.recurrence.is_some() {
                // This unwrap is safe as we have checked that recurrence "is_some"
                #[allow(clippy::unwrap_used)]
                e.set_recurrence(e.recurrence.clone().unwrap())
                    .map_err(|e| {
                        tracing::error!("[update_event] Failed to set recurrence {:?}", e);
                        UseCaseError::InvalidRecurrenceRule
                    })?
            } else {
                e.recurrence = None;
                true
            }
        } else {
            true
        };

        if !valid_recurrence {
            return Err(UseCaseError::InvalidRecurrenceRule);
        };

        if let Some(recurring_event_id_value) = recurring_event_id {
            if recurring_event_id_value.is_some() {
                // Check if the recurring event exists
                e.recurring_event_id = recurring_event_id_value.take();
            } else {
                // Set to NULL
                e.recurring_event_id = None;
            }
        }

        if let Some(original_start_time_value) = original_start_time {
            if original_start_time_value.is_some() {
                e.original_start_time = original_start_time_value.take();
            } else {
                // Set to NULL
                e.original_start_time = None;
            }
        }

        if let Some(title_value) = title {
            if title_value.is_some() {
                e.title = title_value.take();
            } else {
                // Set to NULL
                e.title = None;
            }
        }

        if let Some(description_value) = description {
            if description_value.is_some() {
                e.description = description_value.take();
            } else {
                // Set to NULL
                e.description = None;
            }
        }

        if let Some(event_type_value) = event_type {
            if event_type_value.is_some() {
                e.event_type = event_type_value.take();
            } else {
                // Set to NULL
                e.event_type = None;
            }
        }

        if let Some(external_parent_id_value) = external_parent_id {
            if external_parent_id_value.is_some() {
                e.external_parent_id = external_parent_id_value.take();
            } else {
                // Set to NULL
                e.external_parent_id = None;
            }
        }

        if let Some(external_id_value) = external_id {
            if external_id_value.is_some() {
                e.external_id = external_id_value.take();
            } else {
                // Set to NULL
                e.external_id = None;
            }
        }

        if let Some(location_value) = location {
            if location_value.is_some() {
                e.location = location_value.take();
            } else {
                // Set to NULL
                e.location = None;
            }
        }

        if let Some(status_value) = status {
            e.status = status_value.clone();
        }

        if let Some(all_day_value) = all_day {
            e.all_day = *all_day_value;
        }

        if let Some(created_value) = created {
            e.created = *created_value;
        }

        if let Some(updated_value) = updated {
            e.updated = *updated_value;
        } else {
            // Set to current time
            e.updated = Utc::now();
        }

        ctx.repos
            .events
            .save(&e)
            .await
            .map(|_| e.clone())
            .map_err(|e| {
                tracing::error!("[update_event] Failed to save event {:?}", e);
                UseCaseError::StorageError
            })
    }

    fn subscribers() -> Vec<Box<dyn Subscriber<Self>>> {
        if APP_CONFIG.disable_reminders {
            vec![]
        } else {
            vec![
                Box::new(SyncRemindersOnEventUpdated),
                Box::new(UpdateSyncedEventsOnEventUpdated),
            ]
        }
    }
}

impl PermissionBoundary for UpdateEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::UpdateCalendarEvent]
    }
}

#[cfg(test)]
mod test {
    use nittei_infra::setup_context;

    use super::*;

    #[tokio::test]
    async fn update_nonexisting_event() {
        let mut usecase = UpdateEventUseCase {
            start_time: Some(DateTime::from_timestamp_millis(500).unwrap()),
            duration: Some(800),
            busy: Some(false),
            ..Default::default()
        };
        let ctx = setup_context().await.unwrap();
        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
    }
}
