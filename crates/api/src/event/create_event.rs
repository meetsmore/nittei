use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, TimeDelta, Utc};
use nittei_api_structs::create_event::*;
use nittei_domain::{
    CalendarEvent,
    CalendarEventReminder,
    CalendarEventStatus,
    RRuleOptions,
    User,
    ID,
};
use nittei_infra::NitteiContext;

use super::subscribers::CreateRemindersOnEventCreated;
use crate::{
    error::NitteiError,
    event::subscribers::CreateSyncedEventsOnEventCreated,
    shared::{
        auth::{account_can_modify_user, protect_account_route, protect_route, Permission},
        usecase::{execute, execute_with_policy, PermissionBoundary, Subscriber, UseCase},
    },
};

pub async fn create_event_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body: actix_web_validator::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let body = body.0;
    let usecase = CreateEventUseCase {
        parent_id: body.parent_id,
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
        reminders: body.reminders,
        service_id: body.service_id,
        group_id: body.group_id,
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
        parent_id: body.parent_id,
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
        user,
        reminders: body.reminders,
        service_id: body.service_id,
        group_id: body.group_id,
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
    pub parent_id: Option<String>,
    pub external_id: Option<String>,
    pub location: Option<String>,
    pub status: CalendarEventStatus,
    pub all_day: bool,
    pub start_time: DateTime<Utc>,
    pub duration: i64,
    pub busy: bool,
    pub recurrence: Option<RRuleOptions>,
    pub reminders: Vec<CalendarEventReminder>,
    pub service_id: Option<ID>,
    pub group_id: Option<ID>,
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
    fn from(_: anyhow::Error) -> Self {
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
            parent_id: self.parent_id.clone(),
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
            end_time: self.start_time + TimeDelta::milliseconds(self.duration), // default, if recurrence changes, this will be updated
            exdates: Vec::new(),
            calendar_id: calendar.id.clone(),
            user_id: self.user.id.clone(),
            account_id: self.user.account_id.clone(),
            reminders: self.reminders.clone(),
            service_id: self.service_id.clone(),
            group_id: self.group_id.clone(),
            metadata: self.metadata.clone(),
            created: self.created.unwrap_or_else(Utc::now),
            updated: self.updated.unwrap_or_else(Utc::now),
        };

        if let Some(rrule_opts) = self.recurrence.clone() {
            if !e.set_recurrence(rrule_opts, &calendar.settings, true) {
                return Err(UseCaseError::InvalidRecurrenceRule);
            };
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
        vec![
            Box::new(CreateRemindersOnEventCreated),
            Box::new(CreateSyncedEventsOnEventCreated),
        ]
    }
}

impl PermissionBoundary for CreateEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::CreateCalendarEvent]
    }
}

#[cfg(test)]
mod test {
    use chrono::{prelude::*, Utc};
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

    #[actix_web::main]
    #[test]
    async fn rejects_event_with_invalid_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut invalid_rrules = Vec::new();
        invalid_rrules.push(RRuleOptions {
            count: Some(1000), // too big count
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            until: Some(Utc.with_ymd_and_hms(2150, 1, 1, 0, 0, 0).unwrap()), // too big until
            ..Default::default()
        });
        for rrule in invalid_rrules {
            let mut usecase = CreateEventUseCase {
                start_time: DateTime::from_timestamp_millis(500).unwrap(),
                duration: 800,
                recurrence: Some(rrule),
                calendar_id: calendar.id.clone(),
                user: user.clone(),
                ..Default::default()
            };

            let res = usecase.execute(&ctx).await;

            assert!(res.is_err());
        }
    }
}
