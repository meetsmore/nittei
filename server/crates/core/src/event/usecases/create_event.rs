use crate::event::domain::event::{CalendarEvent, RRuleOptions};
use crate::{
    context::Context,
    shared::{
        auth::Permission,
        usecase::{
            execute, execute_with_policy, PermissionBoundary, UseCase, UseCaseErrorContainer,
        },
    },
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use super::sync_event_reminders::{EventOperation, SyncEventRemindersUseCase};

struct CreateEventUseCase {
    account_id: String,
    calendar_id: String,
    user_id: String,
    start_ts: i64,
    duration: i64,
    busy: Option<bool>,
    rrule_options: Option<RRuleOptions>,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseErrors {
    InvalidRecurrenceRule,
    NotFoundError,
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateEventUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let calendar = match ctx.repos.calendar_repo.find(&self.calendar_id).await {
            Some(calendar) if calendar.user_id == self.user_id => calendar,
            _ => return Err(UseCaseErrors::NotFoundError),
        };

        let mut e = CalendarEvent {
            id: ObjectId::new().to_string(),
            busy: self.busy.unwrap_or(false),
            start_ts: self.start_ts,
            duration: self.duration,
            recurrence: None,
            end_ts: self.start_ts + self.duration, // default, if recurrence changes, this will be updated
            exdates: vec![],
            calendar_id: calendar.id.clone(),
            user_id: self.user_id.clone(),
            account_id: self.account_id.clone(),
            reminder: None,
        };
        if let Some(rrule_opts) = self.rrule_options.clone() {
            if !e.set_recurrence(rrule_opts, &calendar.settings, true) {
                return Err(UseCaseErrors::InvalidRecurrenceRule);
            };
        }

        let repo_res = ctx.repos.event_repo.insert(&e).await;
        if repo_res.is_err() {
            return Err(UseCaseErrors::StorageError);
        }

        let sync_event_reminders = SyncEventRemindersUseCase {
            event: &e,
            op: EventOperation::Created(&calendar),
        };

        // TODO: handl err
        execute(sync_event_reminders, ctx).await;

        Ok(e)
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
    use chrono::Utc;

    use super::*;
    use crate::{calendar::domain::Calendar, user::domain::User};

    struct TestContext {
        ctx: context::Context,
        calendar: Calendar,
        user: User,
    }

    async fn setup() -> TestContext {
        let ctx = Context::create_inmemory();
        let user = User::new("cool2", "cool");

        let calendar = Calendar::new(&user.id);

        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();
        TestContext {
            user,
            calendar,
            ctx,
        }
    }

    #[actix_rt::main]
    #[test]
    async fn creates_event_without_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_ts: 500,
            duration: 800,
            rrule_options: None,
            busy: Some(false),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id,
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[actix_rt::main]
    #[test]
    async fn creates_event_with_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_ts: 500,
            duration: 800,
            rrule_options: Some(Default::default()),
            busy: Some(false),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id,
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[actix_rt::main]
    #[test]
    async fn rejects_invalid_calendar_id() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateEventUseCase {
            start_ts: 500,
            duration: 800,
            rrule_options: Some(Default::default()),
            busy: Some(false),
            calendar_id: format!("1{}", calendar.id),
            user_id: user.id.clone(),
            account_id: user.account_id,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), UseCaseErrors::NotFoundError);
    }

    #[actix_rt::main]
    #[test]
    async fn rejects_event_with_invalid_recurrence() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut invalid_rrules = vec![];
        invalid_rrules.push(RRuleOptions {
            count: Some(1000), // too big count
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            until: Some(Utc.ymd(2150, 1, 1).and_hms(0, 0, 0).timestamp_millis() as isize), // too big until
            ..Default::default()
        });
        for rrule in invalid_rrules {
            let mut usecase = CreateEventUseCase {
                start_ts: 500,
                duration: 800,
                rrule_options: Some(rrule),
                busy: Some(false),
                calendar_id: calendar.id.clone(),
                user_id: user.id.clone(),
                account_id: user.account_id.to_owned(),
            };

            let res = usecase.execute(&ctx).await;

            assert!(res.is_err());
        }
    }
}
