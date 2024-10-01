use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use event::subscribers::SyncRemindersOnEventUpdated;
use nittei_api_structs::update_event::*;
use nittei_domain::{
    CalendarEvent,
    CalendarEventReminder,
    CalendarEventStatus,
    Metadata,
    RRuleOptions,
    User,
    ID,
};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
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
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;
    let user = account_can_modify_user(&account, &e.user_id, &ctx).await?;

    let body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: e.id,
        title: body.title,
        description: body.description,
        parent_id: body.parent_id,
        location: body.location,
        status: body.status,
        all_day: body.all_day,
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
        .map_err(NitteiError::from)
}

pub async fn update_event_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = UpdateEventUseCase {
        user,
        event_id: path_params.event_id.clone(),
        title: body.title,
        description: body.description,
        parent_id: body.parent_id,
        location: body.location,
        status: body.status,
        all_day: body.all_day,
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
        .map_err(NitteiError::from)
}

#[derive(Debug, Default)]
pub struct UpdateEventUseCase {
    pub user: User,
    pub event_id: ID,

    pub title: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub location: Option<String>,
    pub status: Option<CalendarEventStatus>,
    pub all_day: Option<bool>,
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

#[async_trait::async_trait(?Send)]
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
            parent_id,
            location,
            status,
            all_day,
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
            Ok(Some(event)) if event.user_id == user.id => event,
            Ok(_) => {
                return Err(UseCaseError::NotFound(
                    "Calendar Event".into(),
                    event_id.clone(),
                ))
            }
            Err(e) => {
                tracing::error!("Failed to get one event {:?}", e);
                return Err(UseCaseError::StorageError);
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
            Ok(Some(cal)) => cal,
            Ok(None) => {
                return Err(UseCaseError::NotFound(
                    "Calendar".into(),
                    e.calendar_id.clone(),
                ))
            }
            Err(e) => {
                tracing::error!("Failed to get one calendar {:?}", e);
                return Err(UseCaseError::StorageError);
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

        e.title.clone_from(title);

        e.description.clone_from(description);

        e.parent_id.clone_from(parent_id);

        e.location.clone_from(location);

        if let Some(status) = status {
            e.status.clone_from(status);
        }

        if let Some(all_day) = all_day {
            e.all_day = *all_day;
        }

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
    use nittei_infra::setup_context;

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
