use actix_web::{HttpRequest, HttpResponse, web};
use chrono::{DateTime, TimeDelta, Utc};
use nittei_api_structs::create_event::*;
use nittei_domain::{
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
        auth::{Permission, account_can_modify_user, protect_admin_route, protect_route},
        usecase::{PermissionBoundary, Subscriber, UseCase, execute, execute_with_policy},
    },
};

pub async fn create_event_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body: actix_web_validator::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let body = body.0;
    let usecase = CreateEventUseCase {
        external_parent_id: body.external_parent_id,
        external_id: body.external_id,
        title: body.title,
        description: body.description,
        event_type: body.event_type,
        location: body.location,
        status: body.status,
        busy: body.busy.unwrap_or(false),
        all_day: body.all_day.unwrap_or(false),
        start_time: body.start_time,
        duration: body.duration,
        user,
        calendar_id: body.calendar_id,
        recurrence: body.recurrence,
        exdates: body.exdates.unwrap_or_default(),
        recurring_event_id: body.recurring_event_id,
        original_start_time: body.original_start_time,
        reminders: body.reminders,
        service_id: body.service_id,
        metadata: body.metadata,
        created: body.created,
        updated: body.updated,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Created().json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

pub async fn create_event_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = CreateEventUseCase {
        external_parent_id: body.external_parent_id,
        external_id: body.external_id,
        title: body.title,
        description: body.description,
        event_type: body.event_type,
        location: body.location,
        status: body.status,
        busy: body.busy.unwrap_or(false),
        all_day: body.all_day.unwrap_or(false),
        start_time: body.start_time,
        duration: body.duration,
        calendar_id: body.calendar_id,
        recurrence: body.recurrence,
        exdates: body.exdates.unwrap_or_default(),
        recurring_event_id: body.recurring_event_id,
        original_start_time: body.original_start_time,
        user,
        reminders: body.reminders,
        service_id: body.service_id,
        metadata: body.metadata,
        created: body.created,
        updated: body.updated,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Created().json(APIResponse::new(event)))
        .map_err(NitteiError::from)
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

#[async_trait::async_trait(?Send)]
impl UseCase for CreateEventUseCase {
    type Response = CalendarEvent;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateEvent";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let calendar = ctx
            .repos
            .calendars
            .find(&self.calendar_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        let calendar = match calendar {
            Some(calendar) if calendar.user_id == self.user.id => calendar,
            _ => return Err(UseCaseError::NotFound(self.calendar_id.clone())),
        };

        let mut e = CalendarEvent {
            id: Default::default(),
            external_parent_id: self.external_parent_id.clone(),
            external_id: self.external_id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            event_type: self.event_type.clone(),
            location: self.location.clone(),
            status: self.status.clone(),
            all_day: self.all_day,
            busy: self.busy,
            start_time: self.start_time,
            duration: self.duration,
            recurrence: None,
            end_time: self.start_time + TimeDelta::milliseconds(self.duration),
            exdates: self.exdates.clone(),
            recurring_until: None,
            recurring_event_id: self.recurring_event_id.take(),
            original_start_time: self.original_start_time,
            calendar_id: calendar.id.clone(),
            user_id: self.user.id.clone(),
            account_id: self.user.account_id.clone(),
            reminders: self.reminders.clone(),
            service_id: self.service_id.take(),
            metadata: self.metadata.take(),
            created: self.created.unwrap_or_else(Utc::now),
            updated: self.updated.unwrap_or_else(Utc::now),
        };

        // If we have recurrence, check if it's valid and set it
        if let Some(rrule_opts) = self.recurrence.clone() {
            let res = e.set_recurrence(rrule_opts).map_err(|e| {
                tracing::error!("[create_event] Error setting recurrence: {:?}", e);
                UseCaseError::InvalidRecurrenceRule
            })?;
            if !res {
                return Err(UseCaseError::InvalidRecurrenceRule);
            }
        }

        // TODO: maybe have reminders length restriction
        for reminder in &self.reminders {
            if !reminder.is_valid() {
                return Err(UseCaseError::InvalidReminder);
            }
        }

        ctx.repos.events.insert(&e).await?;

        Ok(e)
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

impl PermissionBoundary for CreateEventUseCase {
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

    #[actix_web::main]
    #[test]
    async fn creates_event_without_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_time: DateTime::from_timestamp_millis(500).unwrap(),
            duration: 800,
            calendar_id: calendar.id.clone(),
            user,
            ..Default::default()
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn creates_event_with_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_time: DateTime::from_timestamp_millis(500).unwrap(),
            duration: 800,
            recurrence: Some(Default::default()),
            calendar_id: calendar.id.clone(),
            user,
            ..Default::default()
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_invalid_calendar_id() {
        let TestContext {
            ctx,
            calendar: _,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_time: DateTime::from_timestamp_millis(500).unwrap(),
            duration: 800,
            recurrence: Some(Default::default()),
            user,
            ..Default::default()
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            UseCaseError::NotFound(usecase.calendar_id)
        );
    }
}
