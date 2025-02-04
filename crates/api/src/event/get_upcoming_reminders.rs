use std::{collections::HashMap, time::Duration};

use actix_web::rt::time::Instant;
use chrono::TimeDelta;
use nittei_api_structs::send_event_reminders::{AccountEventReminder, AccountReminders};
use nittei_domain::{Account, CalendarEvent, Reminder};
use nittei_infra::NitteiContext;
use tracing::error;

use crate::{error::NitteiError, shared::usecase::UseCase};

/// Creates EventReminders for a calendar event
#[derive(Debug)]
pub struct GetUpcomingRemindersUseCase {
    /// Will fetch reminders for this interval
    pub reminders_interval: i64,
}

#[derive(Debug)]
pub enum UseCaseError {
    IntervalServerError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::IntervalServerError => Self::InternalError,
        }
    }
}

async fn get_accounts_from_reminders(
    reminders: &[Reminder],
    ctx: &NitteiContext,
) -> anyhow::Result<HashMap<String, Account>> {
    let account_ids: Vec<_> = reminders
        .iter()
        .map(|r| r.account_id.to_owned())
        .collect::<Vec<_>>();
    Ok(ctx
        .repos
        .accounts
        .find_many(&account_ids)
        .await?
        .into_iter()
        .map(|a| (a.id.to_string(), a))
        .collect())
}

async fn create_reminders_for_accounts(
    reminders: Vec<Reminder>,
    event_lookup: HashMap<String, CalendarEvent>,
    ctx: &NitteiContext,
) -> anyhow::Result<Vec<(Account, AccountReminders)>> {
    let account_lookup = get_accounts_from_reminders(&reminders, ctx).await?;

    let mut account_reminders: HashMap<String, (&Account, AccountReminders)> = HashMap::new();

    for reminder in reminders {
        let account = match account_lookup.get(&reminder.account_id.to_string()) {
            Some(a) => a,
            None => continue,
        };

        // Remove instead of get because there shouldn't be multiple reminders for the same event id
        // and also we get ownership over calendar_event
        let calendar_event = match event_lookup.get(&reminder.event_id.to_string()) {
            Some(e) => e.clone(),
            None => continue,
        };
        match account_reminders.get_mut(&account.id.to_string()) {
            Some(acc_reminders) => {
                acc_reminders.1.reminders.push(AccountEventReminder {
                    event: calendar_event,
                    identifier: reminder.identifier,
                });
            }
            None => {
                account_reminders.insert(
                    account.id.to_string(),
                    (
                        account,
                        AccountReminders {
                            reminders: vec![AccountEventReminder {
                                event: calendar_event,
                                identifier: reminder.identifier,
                            }],
                        },
                    ),
                );
            }
        };
    }

    Ok(account_reminders
        .into_iter()
        .map(|(_, (acc, reminders))| (acc.clone(), reminders))
        .collect())
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetUpcomingRemindersUseCase {
    type Response = (Vec<(Account, AccountReminders)>, Instant);

    type Error = UseCaseError;

    const NAME: &'static str = "GetUpcomingReminders";

    /// This will run every minute
    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        // Find all occurrences for the next interval and delete them
        let ts = ctx.sys.get_timestamp() + TimeDelta::milliseconds(self.reminders_interval);

        // Get all reminders and filter out invalid / expired reminders
        let reminders = ctx
            .repos
            .reminders
            .delete_all_before(ts)
            .await
            .map_err(|_| UseCaseError::IntervalServerError)?;

        let event_lookup = ctx
            .repos
            .events
            .find_many(
                &reminders
                    .iter()
                    .map(|r| r.event_id.clone())
                    .collect::<Vec<_>>(),
            )
            .await
            .map_err(|e| {
                error!("{:?}", e);
                UseCaseError::IntervalServerError
            })?
            .into_iter()
            .map(|e| (e.id.to_string(), e))
            .collect::<HashMap<_, _>>();

        let grouped_reminders = create_reminders_for_accounts(reminders, event_lookup, ctx)
            .await
            .map_err(|e| {
                error!("{:?}", e);
                UseCaseError::IntervalServerError
            })?;

        let millis_to_send = ts.timestamp_millis() - ctx.sys.get_timestamp_millis();
        let instant = if millis_to_send > 0 {
            Instant::now() + Duration::from_millis(millis_to_send as u64)
        } else {
            error!("Important: Increase computation time for get reminders usecase");
            Instant::now()
        };
        Ok((grouped_reminders, instant))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{DateTime, Utc};
    use nittei_domain::{Calendar, CalendarEventReminder, User};
    use nittei_infra::{setup_context as _setup_ctx, ISys};

    use super::{super::create_event::CreateEventUseCase, *};
    use crate::{
        event::{delete_event::DeleteEventUseCase, update_event::UpdateEventUseCase},
        shared::usecase::execute,
    };

    async fn setup_context() -> NitteiContext {
        let ctx = _setup_ctx().await.unwrap();
        ctx.repos
            .reminders
            .delete_all_before(DateTime::<Utc>::MAX_UTC)
            .await
            .unwrap();

        ctx
    }

    pub struct StaticTimeSys1;
    impl ISys for StaticTimeSys1 {
        fn get_timestamp_millis(&self) -> i64 {
            1613862000000 // Sun Feb 21 2021 00:00:00 GMT+0100 (Central European Standard Time) {}
        }
        fn get_timestamp(&self) -> DateTime<Utc> {
            DateTime::from_timestamp_millis(1613862000000).unwrap()
        }
    }

    pub struct StaticTimeSys2;
    impl ISys for StaticTimeSys2 {
        fn get_timestamp_millis(&self) -> i64 {
            1613862000000 + 1000 * 60 * 49 // Sun Feb 21 2021 00:49:00 GMT+0100 (Central European Standard Time) {}
        }
        fn get_timestamp(&self) -> DateTime<Utc> {
            DateTime::from_timestamp_millis(1613862000000 + 1000 * 60 * 49).unwrap()
        }
    }

    pub struct StaticTimeSys3;
    impl ISys for StaticTimeSys3 {
        fn get_timestamp_millis(&self) -> i64 {
            1613862000000 + 1000 * 60 * 60 * 24 // Sun Feb 22 2021 00:00:00 GMT+0100 (Central European Standard Time) {}
        }
        fn get_timestamp(&self) -> DateTime<Utc> {
            DateTime::from_timestamp_millis(1613862000000 + 1000 * 60 * 60 * 24).unwrap()
        }
    }

    async fn insert_common_data(ctx: &NitteiContext) -> (User, Calendar) {
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();

        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let mut calendar = Calendar::new(&user.id, &account.id, None, None);
        calendar.settings.timezone = chrono_tz::Europe::Oslo;
        ctx.repos.calendars.insert(&calendar).await.unwrap();
        (user, calendar)
    }

    #[actix_web::main]
    #[serial_test::serial]
    #[test]
    async fn serial_get_upcoming_reminders() {
        let mut ctx = setup_context().await;
        ctx.sys = Arc::new(StaticTimeSys1 {});

        let (user, calendar) = insert_common_data(&ctx).await;

        let usecase = CreateEventUseCase {
            user: user.clone(),
            calendar_id: calendar.id.clone(),
            start_time: ctx.sys.get_timestamp(),
            duration: 1000 * 60 * 60 * 2,
            recurrence: Some(Default::default()),
            reminders: vec![
                CalendarEventReminder {
                    delta: -10, // 10 minutes before start
                    identifier: "".into(),
                },
                CalendarEventReminder {
                    delta: 10, // 10 minutes after start
                    identifier: "".into(),
                },
            ],
            ..Default::default()
        };

        execute(usecase, &ctx).await.unwrap();

        let sys3 = StaticTimeSys3 {};
        let usecase = CreateEventUseCase {
            calendar_id: calendar.id.clone(),
            user,
            start_time: sys3.get_timestamp() + TimeDelta::milliseconds(1000 * 60 * 5),
            duration: 1000 * 60 * 60 * 2,
            reminders: vec![CalendarEventReminder {
                delta: -10, // 10 minutes before start
                identifier: "".into(),
            }],
            ..Default::default()
        };

        execute(usecase, &ctx).await.unwrap();

        ctx.sys = Arc::new(StaticTimeSys1 {});
        let usecase = GetUpcomingRemindersUseCase {
            reminders_interval: 1000 * 60, // 1 minute in milliseconds
        };
        let res = execute(usecase, &ctx).await;
        assert!(res.is_ok());
        let res = res.unwrap().0;
        assert_eq!(res.len(), 0);

        ctx.sys = Arc::new(StaticTimeSys2 {});
        let usecase = GetUpcomingRemindersUseCase {
            reminders_interval: 1000 * 60,
        };
        let res = execute(usecase, &ctx).await;
        assert!(res.is_ok());
        let res = res.unwrap().0;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1.reminders.len(), 1);

        ctx.sys = Arc::new(StaticTimeSys3 {});
        let usecase = GetUpcomingRemindersUseCase {
            reminders_interval: 1000 * 60,
        };
        let res = execute(usecase, &ctx).await;
        assert!(res.is_ok());
        let res = res.unwrap().0;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1.reminders.len(), 2);

        let usecase = GetUpcomingRemindersUseCase {
            reminders_interval: 1000 * 60,
        };
        let res = execute(usecase, &ctx).await;
        assert!(res.is_ok());
        let res = res.unwrap().0;
        assert_eq!(res.len(), 0);
    }

    #[actix_web::main]
    #[serial_test::serial]
    #[test]
    async fn serial_updating_event_also_updates_reminders() {
        let mut ctx = setup_context().await;
        ctx.sys = Arc::new(StaticTimeSys1 {});

        let now = ctx.sys.get_timestamp();
        let initial_start_time = now + TimeDelta::milliseconds(30 * 60 * 1000);
        let delta = -10;

        let (user, calendar) = insert_common_data(&ctx).await;
        let usecase = CreateEventUseCase {
            calendar_id: calendar.id.clone(),
            user: user.clone(),
            start_time: initial_start_time,
            duration: 1000 * 60 * 60 * 2,
            recurrence: Some(Default::default()),
            reminders: vec![CalendarEventReminder {
                delta,
                identifier: "".into(),
            }],
            ..Default::default()
        };

        let calendar_event = execute(usecase, &ctx).await.unwrap();
        let old_reminders = ctx
            .repos
            .reminders
            .delete_all_before(initial_start_time)
            .await
            .unwrap();
        ctx.repos
            .reminders
            .bulk_insert(&old_reminders)
            .await
            .unwrap();

        let start_ts_diff = TimeDelta::milliseconds(15 * 60 * 1000); // 15 minutes
        let new_start = calendar_event.start_time + start_ts_diff; // Postponed 15 minutes
        let user = ctx.repos.users.find(&user.id).await.unwrap().unwrap();
        let update_event_usecase = UpdateEventUseCase {
            event_id: calendar_event.id,
            user,
            reminders: Some(vec![CalendarEventReminder {
                delta,
                identifier: "".into(),
            }]),
            recurrence: Some(Default::default()),
            start_time: Some(new_start),
            ..Default::default()
        };
        execute(update_event_usecase, &ctx).await.unwrap();
        let new_reminders = ctx
            .repos
            .reminders
            .delete_all_before(new_start)
            .await
            .unwrap();
        assert_eq!(new_reminders.len(), old_reminders.len());
        assert_eq!(new_reminders.len(), 1);
        assert_eq!(
            new_reminders[0].remind_at,
            new_start + TimeDelta::milliseconds(delta * 60 * 1000)
        );
        assert_eq!(new_reminders[0].event_id, old_reminders[0].event_id);
        assert_eq!(
            new_reminders[0].remind_at,
            old_reminders[0].remind_at + start_ts_diff
        );
    }

    #[actix_web::main]
    #[serial_test::serial]
    #[test]
    async fn serial_deleting_event_reminder_setting_also_deletes_reminders() {
        let mut ctx = setup_context().await;
        ctx.sys = Arc::new(StaticTimeSys1 {});

        let now = ctx.sys.get_timestamp();
        let delta = 120;
        let remind_at = now + TimeDelta::milliseconds(120 * 60 * 1000);

        let (user, calendar) = insert_common_data(&ctx).await;
        let usecase = CreateEventUseCase {
            user: user.clone(),
            calendar_id: calendar.id.clone(),
            start_time: now,
            duration: 1000 * 60 * 60 * 2,
            recurrence: Some(Default::default()),
            reminders: vec![CalendarEventReminder {
                delta,
                identifier: "".into(),
            }],
            ..Default::default()
        };

        let calendar_event = execute(usecase, &ctx).await.unwrap();
        let old_reminders = ctx
            .repos
            .reminders
            .delete_all_before(remind_at)
            .await
            .unwrap();
        ctx.repos
            .reminders
            .bulk_insert(&old_reminders)
            .await
            .unwrap();
        let acc_reminders = old_reminders
            .into_iter()
            .filter(|r| r.event_id == calendar_event.id)
            .collect::<Vec<_>>();
        assert_eq!(acc_reminders.len(), 1);

        let update_event_usecase = UpdateEventUseCase {
            user: user.clone(),
            event_id: calendar_event.id.clone(),
            reminders: Some(Vec::new()),
            recurrence: Some(Default::default()),
            ..Default::default()
        };
        execute(update_event_usecase, &ctx).await.unwrap();
        let new_reminders = ctx
            .repos
            .reminders
            .delete_all_before(remind_at)
            .await
            .unwrap();
        let acc_reminders = new_reminders
            .into_iter()
            .filter(|r| r.event_id == calendar_event.id)
            .collect::<Vec<_>>();

        assert!(acc_reminders.is_empty());
    }

    #[actix_web::main]
    #[serial_test::serial]
    #[test]
    async fn serial_deleting_event_also_deletes_reminders() {
        let mut ctx = setup_context().await;
        ctx.sys = Arc::new(StaticTimeSys1 {});

        let now = ctx.sys.get_timestamp();

        let (user, calendar) = insert_common_data(&ctx).await;
        let delta = 120;
        let remind_at = now + TimeDelta::milliseconds(delta * 60 * 1000);
        let usecase = CreateEventUseCase {
            user: user.clone(),
            calendar_id: calendar.id.clone(),
            start_time: now,
            duration: 1000 * 60 * 60 * 2,
            recurrence: Some(Default::default()),
            reminders: vec![CalendarEventReminder {
                delta,
                identifier: "".into(),
            }],
            ..Default::default()
        };

        let calendar_event = execute(usecase, &ctx).await.unwrap();
        let old_reminders = ctx
            .repos
            .reminders
            .delete_all_before(remind_at)
            .await
            .unwrap();

        ctx.repos
            .reminders
            .bulk_insert(&old_reminders)
            .await
            .unwrap();
        let acc_reminders = old_reminders
            .into_iter()
            .filter(|r| r.event_id == calendar_event.id)
            .collect::<Vec<_>>();
        assert_eq!(acc_reminders.len(), 1);

        let update_event_usecase = DeleteEventUseCase {
            user,
            event_id: calendar_event.id,
        };
        execute(update_event_usecase, &ctx).await.unwrap();
        let new_reminders = ctx
            .repos
            .reminders
            .delete_all_before(remind_at)
            .await
            .unwrap();
        assert!(new_reminders.is_empty());
    }
}
