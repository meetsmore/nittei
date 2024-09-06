use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use event::subscribers::SyncRemindersOnEventUpdated;
use nettu_scheduler_api_structs::update_event::*;
use nettu_scheduler_domain::{
    CalendarEvent,
    CalendarEventReminder,
    Metadata,
    RRuleOptions,
    User,
    ID,
};
use nettu_scheduler_infra::NettuContext;

use crate::{
    error::NettuError,
    event::{self, subscribers::UpdateSyncedEventsOnEventUpdated},
    shared::{
        auth::{
            account_can_modify_event,
            account_can_modify_user,
            protect_account_route,
            protect_route,
            Permission,
        },
        usecase::{execute, execute_with_policy, PermissionBoundary, Subscriber, UseCase},
    },
};

pub async fn update_event_admin_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;
    let user = account_can_modify_user(&account, &e.user_id, &ctx).await?;

    let body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: e.id,
        duration: body.duration,
        start_time: body.start_time,
        reminders: body.reminders,
        recurrence: body.recurrence,
        busy: body.busy,
        service_id: body.service_id,
        exdates: body.exdates,
        metadata: body.metadata,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(NettuError::from)
}

pub async fn update_event_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: path_params.event_id.clone(),
        duration: body.duration,
        start_time: body.start_time,
        reminders: body.reminders,
        recurrence: body.recurrence,
        busy: body.busy,
        service_id: body.service_id,
        exdates: body.exdates,
        metadata: body.metadata,
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(NettuError::from)
}

#[derive(Debug, Default)]
pub struct UpdateEventUseCase {
    pub user: User,
    pub event_id: ID,
    pub start_time: Option<DateTime<Utc>>,
    pub busy: Option<bool>,
    pub duration: Option<i64>,
    pub reminders: Option<Vec<CalendarEventReminder>>,
    pub recurrence: Option<RRuleOptions>,
    pub service_id: Option<ID>,
    pub exdates: Option<Vec<DateTime<Utc>>>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(String, ID),
    InvalidReminder,
    StorageError,
    InvalidRecurrenceRule,
}

impl From<UseCaseError> for NettuError {
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

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateEventUseCase {
    type Response = CalendarEvent;

    type Error = UseCaseError;

    const NAME: &'static str = "UpdateEvent";

    async fn execute(&mut self, ctx: &NettuContext) -> Result<Self::Response, Self::Error> {
        let UpdateEventUseCase {
            user,
            event_id,
            start_time,
            busy,
            duration,
            recurrence,
            exdates,
            reminders,
            service_id,
            metadata,
        } = self;

        let mut e = match ctx.repos.events.find(event_id).await {
            Some(event) if event.user_id == user.id => event,
            _ => {
                return Err(UseCaseError::NotFound(
                    "Calendar Event".into(),
                    event_id.clone(),
                ))
            }
        };

        e.service_id.clone_from(service_id);

        if let Some(exdates) = exdates {
            e.exdates.clone_from(exdates);
        }
        if let Some(metadata) = metadata {
            e.metadata = metadata.clone();
        }

        if let Some(reminders) = &reminders {
            for reminder in reminders {
                if !reminder.is_valid() {
                    return Err(UseCaseError::InvalidReminder);
                }
            }
            e.reminders.clone_from(reminders);
        }

        let calendar = match ctx.repos.calendars.find(&e.calendar_id).await {
            Some(cal) => cal,
            _ => {
                return Err(UseCaseError::NotFound(
                    "Calendar".into(),
                    e.calendar_id.clone(),
                ))
            }
        };

        let mut start_or_duration_change = false;

        if let Some(start_time) = start_time {
            if e.start_time != *start_time {
                e.start_time = *start_time;
                e.exdates = Vec::new();
                start_or_duration_change = true;
            }
        }
        if let Some(duration) = duration {
            if e.duration != *duration {
                e.duration = *duration;
                start_or_duration_change = true;
            }
        }
        if let Some(busy) = busy {
            e.busy = *busy;
        }

        let valid_recurrence = if let Some(rrule_opts) = recurrence.clone() {
            // ? should exdates be deleted when rrules are updated
            e.set_recurrence(rrule_opts, &calendar.settings, true)
        } else if start_or_duration_change && e.recurrence.is_some() {
            // This unwrap is safe as we have checked that recurrence "is_some"
            #[allow(clippy::unwrap_used)]
            e.set_recurrence(e.recurrence.clone().unwrap(), &calendar.settings, true)
        } else {
            e.recurrence = None;
            true
        };

        if !valid_recurrence {
            return Err(UseCaseError::InvalidRecurrenceRule);
        };

        e.updated = ctx.sys.get_timestamp_millis();

        ctx.repos
            .events
            .save(&e)
            .await
            .map(|_| e.clone())
            .map_err(|_| UseCaseError::StorageError)
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
    use nettu_scheduler_infra::setup_context;

    use super::*;

    #[actix_web::main]
    #[test]
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
