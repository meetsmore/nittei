use axum::{Extension, Json, extract::Path, http::StatusCode};
use axum_valid::Valid;
use chrono::{DateTime, TimeDelta, Utc};
use nittei_api_structs::create_many_events::*;
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

use super::subscribers::CreateRemindersOnEventCreated;
use crate::{
    error::NitteiError,
    event::subscribers::CreateSyncedEventsOnEventCreated,
    shared::{
        auth::{Permission, account_can_modify_user},
        usecase::{PermissionBoundary, Subscriber, UseCase, execute},
    },
};

#[utoipa::path(
    post,
    tag = "Event",
    path = "/api/v1/user/{user_id}/events/create_many",
    summary = "Create many events (admin only)",
    params(
        ("user_id" = ID, Path, description = "The id of the user to create the events for"),
    ),
    security(
        ("api_key" = [])
    ),
    request_body(
        content = CreateManyEventsRequestBody,
    ),
    responses(
        (status = 200, body = CreateManyEventsAPIResponse)
    )
)]
pub async fn create_many_events_admin_controller(
    Extension(account): Extension<Account>,
    path_params: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
    Valid(Json(body)): Valid<Json<CreateManyEventsRequestBody>>,
) -> Result<(StatusCode, Json<CreateManyEventsAPIResponse>), NitteiError> {
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let usecase = CreateManyEventsUseCase {
        events: body
            .events
            .into_iter()
            .map(|mut e| CreateEventUseCase {
                external_parent_id: e.external_parent_id.take(),
                external_id: e.external_id.take(),
                title: e.title.take(),
                description: e.description.take(),
                event_type: e.event_type.take(),
                location: e.location.take(),
                status: e.status.clone(),
                busy: e.busy.unwrap_or(false),
                all_day: e.all_day.unwrap_or(false),
                start_time: e.start_time,
                duration: e.duration,
                user: user.clone(),
                calendar_id: e.calendar_id.clone(),
                recurrence: e.recurrence.take(),
                exdates: e.exdates.clone().unwrap_or_default(),
                recurring_event_id: e.recurring_event_id.take(),
                original_start_time: e.original_start_time,
                reminders: e.reminders.clone(),
                service_id: e.service_id.take(),
                metadata: e.metadata.take(),
                created: e.created,
                updated: e.updated,
            })
            .collect(),
    };

    execute(usecase, &ctx)
        .await
        .map(|events| {
            (
                StatusCode::CREATED,
                Json(CreateManyEventsAPIResponse::new(events)),
            )
        })
        .map_err(NitteiError::from)
}

#[derive(Debug, Default)]
pub struct CreateManyEventsUseCase {
    pub events: Vec<CreateEventUseCase>,
}

#[derive(Debug, Default)]
pub struct CreateEventUseCase {
    pub calendar_id: ID,
    pub user: User,
    pub title: Option<String>,
    pub description: Option<String>,
    pub event_type: Option<String>,
    pub external_parent_id: Option<String>,
    pub external_id: Option<String>,
    pub location: Option<String>,
    pub status: CalendarEventStatus,
    pub all_day: bool,
    pub start_time: DateTime<Utc>,
    pub duration: i64,
    pub busy: bool,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<DateTime<Utc>>,
    pub recurring_event_id: Option<ID>,
    pub original_start_time: Option<DateTime<Utc>>,
    pub reminders: Vec<CalendarEventReminder>,
    pub service_id: Option<ID>,
    pub metadata: Option<serde_json::Value>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseError {
    InvalidRecurrenceRule,
    InvalidReminder,
    NotFound(ID),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(calendar_id) => Self::NotFound(format!(
                "The calendar with id: {}, was not found.",
                calendar_id
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

impl From<anyhow::Error> for UseCaseError {
    fn from(error: anyhow::Error) -> Self {
        tracing::error!("[create_event] Unexpected error: {:?}", error);
        UseCaseError::StorageError
    }
}

#[async_trait::async_trait]
impl UseCase for CreateManyEventsUseCase {
    type Response = Vec<CalendarEvent>;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateManyEvents";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        // Collect calendar ids from events, and avoid duplicates
        let calendar_ids: Vec<&ID> = self
            .events
            .iter()
            .map(|e| &e.calendar_id)
            .collect::<Vec<_>>();

        // Find calendars by ids
        let calendars = ctx
            .repos
            .calendars
            .find_multiple(calendar_ids)
            .await
            .map_err(|_| UseCaseError::StorageError)?;

        let mut events = Vec::new();
        // Create events
        for event in &self.events {
            let calendar = calendars
                .iter()
                .find(|c| c.id == event.calendar_id)
                .ok_or(UseCaseError::NotFound(event.calendar_id.clone()))?;
            if calendar.user_id != event.user.id {
                return Err(UseCaseError::NotFound(event.calendar_id.clone()));
            }

            let mut e = CalendarEvent {
                id: Default::default(),
                external_parent_id: event.external_parent_id.clone(),
                external_id: event.external_id.clone(),
                title: event.title.clone(),
                description: event.description.clone(),
                event_type: event.event_type.clone(),
                location: event.location.clone(),
                status: event.status.clone(),
                all_day: event.all_day,
                busy: event.busy,
                start_time: event.start_time,
                duration: event.duration,
                recurrence: None,
                end_time: event.start_time + TimeDelta::milliseconds(event.duration),
                exdates: event.exdates.clone(),
                recurring_until: None,
                recurring_event_id: event.recurring_event_id.clone(),
                original_start_time: event.original_start_time,
                calendar_id: calendar.id.clone(),
                user_id: event.user.id.clone(),
                account_id: event.user.account_id.clone(),
                reminders: event.reminders.clone(),
                service_id: event.service_id.clone(),
                metadata: event.metadata.clone(),
                created: event.created.unwrap_or_else(Utc::now),
                updated: event.updated.unwrap_or_else(Utc::now),
            };

            // If we have recurrence, check if it's valid and set it
            if let Some(rrule_opts) = event.recurrence.clone() {
                let res = e.set_recurrence(rrule_opts).map_err(|e| {
                    tracing::error!("[create_event] Error setting recurrence: {:?}", e);
                    UseCaseError::InvalidRecurrenceRule
                })?;
                if !res {
                    return Err(UseCaseError::InvalidRecurrenceRule);
                }
            }

            // TODO: maybe have reminders length restriction
            for reminder in &event.reminders {
                if !reminder.is_valid() {
                    return Err(UseCaseError::InvalidReminder);
                }
            }

            events.push(e);
        }

        // Create events
        for event in &events {
            ctx.repos
                .events
                .insert(event)
                .await
                .map_err(|_| UseCaseError::StorageError)?;
        }

        Ok(events)
    }

    fn subscribers() -> Vec<Box<dyn Subscriber<Self>>> {
        if APP_CONFIG.disable_reminders {
            vec![]
        } else {
            vec![
                Box::new(CreateRemindersOnEventCreated),
                Box::new(CreateSyncedEventsOnEventCreated),
            ]
        }
    }
}

impl PermissionBoundary for CreateManyEventsUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateCalendarEvent]
    }
}

#[cfg(test)]
mod test {
    use chrono::prelude::*;
    use nittei_domain::{Account, Calendar, User};
    use nittei_infra::setup_context;

    use super::*;

    struct TestContext {
        ctx: NitteiContext,
        calendar: Calendar,
        user: User,
    }

    async fn setup() -> TestContext {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id, None, None);
        ctx.repos.calendars.insert(&calendar).await.unwrap();

        TestContext {
            user,
            calendar,
            ctx,
        }
    }

    #[tokio::test]
    async fn creates_event_without_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateManyEventsUseCase {
            events: vec![CreateEventUseCase {
                start_time: DateTime::from_timestamp_millis(500).unwrap(),
                duration: 800,
                calendar_id: calendar.id.clone(),
                user,
                ..Default::default()
            }],
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn creates_event_with_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateManyEventsUseCase {
            events: vec![CreateEventUseCase {
                start_time: DateTime::from_timestamp_millis(500).unwrap(),
                duration: 800,
                recurrence: Some(Default::default()),
                calendar_id: calendar.id.clone(),
                user,
                ..Default::default()
            }],
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn rejects_invalid_calendar_id() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateManyEventsUseCase {
            events: vec![CreateEventUseCase {
                start_time: DateTime::from_timestamp_millis(500).unwrap(),
                duration: 800,
                recurrence: Some(Default::default()),
                user,
                ..Default::default()
            }],
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            UseCaseError::NotFound(calendar.id.clone())
        );
    }
}
