use axum::{
    Extension,
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use axum_valid::Valid;
use chrono::{DateTime, TimeDelta, Utc};
use event::subscribers::SyncRemindersOnEventUpdated;
use nittei_api_structs::update_event::*;
use nittei_domain::{
    CalendarEvent,
    CalendarEventReminder,
    CalendarEventStatus,
    ID,
    RRuleOptions,
    User,
};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    event::{self, subscribers::UpdateSyncedEventsOnEventUpdated},
    shared::{
        auth::{
            Permission,
            account_can_modify_event,
            account_can_modify_user,
            protect_admin_route,
            protect_route,
        },
        usecase::{PermissionBoundary, Subscriber, UseCase, execute, execute_with_policy},
    },
};

pub async fn update_event_admin_controller(
    headers: HeaderMap,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
    body: Valid<Json<RequestBody>>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;
    let user = account_can_modify_user(&account, &e.user_id, &ctx).await?;

    let mut body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: e.id,
        title: body.title.take(),
        description: body.description.take(),
        event_type: body.event_type.take(),
        external_parent_id: body.parent_id.take(),
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
    };

    execute(usecase, &ctx)
        .await
        .map(|event| Json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

pub async fn update_event_controller(
    headers: HeaderMap,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
    body: Valid<Json<RequestBody>>,
) -> Result<Json<APIResponse>, NitteiError> {
    let (user, policy) = protect_route(&headers, &ctx).await?;

    let mut body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: path_params.event_id.clone(),
        title: body.title.take(),
        description: body.description.take(),
        event_type: body.event_type.take(),
        external_parent_id: body.parent_id.take(),
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
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| Json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

#[derive(Debug, Default)]
pub struct UpdateEventUseCase {
    pub user: User,
    pub event_id: ID,

    pub title: Option<String>,
    pub description: Option<String>,
    pub event_type: Option<String>,
    pub external_parent_id: Option<String>,
    pub external_id: Option<String>,
    pub location: Option<String>,
    pub status: Option<CalendarEventStatus>,
    pub all_day: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub busy: Option<bool>,
    pub duration: Option<i64>,
    pub reminders: Option<Vec<CalendarEventReminder>>,
    pub service_id: Option<ID>,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Option<Vec<DateTime<Utc>>>,
    pub recurring_event_id: Option<ID>,
    pub original_start_time: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
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
            UseCaseError::NotFound(entity, event_id) => Self::NotFound(format!(
                "The {} with id: {}, was not found.",
                entity, event_id
            )),
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
        } = self;

        let mut e = match ctx.repos.events.find(event_id).await {
            Ok(Some(event)) if event.user_id == user.id => event,
            Ok(_) => {
                return Err(UseCaseError::NotFound(
                    "Calendar Event".into(),
                    event_id.clone(),
                ));
            }
            Err(e) => {
                tracing::error!("Failed to get one event {:?}", e);
                return Err(UseCaseError::StorageError);
            }
        };

        if service_id.is_some() {
            e.service_id.clone_from(service_id);
        }

        if let Some(exdates) = exdates {
            e.exdates.clone_from(exdates);
        }
        if let Some(metadata) = metadata {
            e.metadata = Some(metadata.clone());
        }

        if let Some(reminders) = &reminders {
            for reminder in reminders {
                if !reminder.is_valid() {
                    return Err(UseCaseError::InvalidReminder);
                }
            }
            e.reminders.clone_from(reminders);
        }

        let mut start_or_duration_change = false;

        if let Some(start_time) = start_time {
            // Only change the exdates if the start time has actually changed
            if e.start_time != *start_time {
                e.exdates = Vec::new();
            }
            e.start_time = *start_time;
            start_or_duration_change = true;
        }
        if let Some(duration) = duration {
            e.duration = *duration;
            start_or_duration_change = true;
        }

        if start_or_duration_change {
            e.end_time = e.start_time + TimeDelta::milliseconds(e.duration);
        }

        if let Some(busy) = busy {
            e.busy = *busy;
        }

        // Handle the new recurrence
        let valid_recurrence = if let Some(rrule_opts) = recurrence.clone() {
            // ? should exdates be deleted when rrules are updated
            e.set_recurrence(rrule_opts).map_err(|e| {
                tracing::error!("[update_event] Failed to set recurrence {:?}", e);
                UseCaseError::InvalidRecurrenceRule
            })?

        // Otherwise, we we don't have a new recurrence, but we have an existing one
        // And the start time or duration has changed, we need to update the recurrence
        } else if start_or_duration_change && e.recurrence.is_some() {
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
        };

        if !valid_recurrence {
            return Err(UseCaseError::InvalidRecurrenceRule);
        };

        if let Some(recurring_event_id) = recurring_event_id {
            // Check if the recurring event exists
            e.recurring_event_id = Some(recurring_event_id.clone());
        }

        if let Some(original_start_time) = original_start_time {
            e.original_start_time = Some(*original_start_time);
        }

        if title.is_some() {
            e.title.clone_from(title);
        }

        if description.is_some() {
            e.description.clone_from(description);
        }

        if event_type.is_some() {
            e.event_type.clone_from(event_type);
        }

        if external_parent_id.is_some() {
            e.external_parent_id.clone_from(external_parent_id);
        }

        if external_id.is_some() {
            e.external_id.clone_from(external_id);
        }

        if location.is_some() {
            e.location.clone_from(location);
        }

        if let Some(status) = status {
            e.status = status.clone();
        }

        if let Some(all_day) = all_day {
            e.all_day = *all_day;
        }

        if let Some(created) = created {
            e.created = *created;
        }

        if let Some(updated) = updated {
            e.updated = *updated;
        } else {
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
        vec![
            Box::new(SyncRemindersOnEventUpdated),
            Box::new(UpdateSyncedEventsOnEventUpdated),
        ]
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
