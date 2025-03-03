mod postgres;

use chrono::{DateTime, Utc};
use nittei_domain::EventRemindersExpansionJob;
pub use postgres::PostgresEventReminderGenerationJobsRepo;

#[async_trait::async_trait]
pub trait IEventRemindersGenerationJobsRepo: Send + Sync {
    async fn bulk_insert(&self, jobs: &[EventRemindersExpansionJob]) -> anyhow::Result<()>;
    async fn delete_all_before(
        &self,
        before: DateTime<Utc>,
    ) -> anyhow::Result<Vec<EventRemindersExpansionJob>>;
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use nittei_domain::{Account, Calendar, CalendarEvent, EventRemindersExpansionJob, User};
    use tracing::error;

    use crate::setup_context;

    #[tokio::test]
    async fn crud() {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id, None, None);
        ctx.repos.calendars.insert(&calendar).await.unwrap();
        let e1 = CalendarEvent {
            account_id: account.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60,
            start_time: DateTime::from_timestamp_millis(1000 * 60 * 60).unwrap(),
            user_id: user.id.clone(),
            ..Default::default()
        };
        ctx.repos.events.insert(&e1).await.unwrap();
        let e2 = CalendarEvent {
            account_id: account.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60,
            start_time: DateTime::from_timestamp_millis(1000 * 60 * 60).unwrap(),
            user_id: user.id.clone(),
            ..Default::default()
        };
        ctx.repos.events.insert(&e2).await.unwrap();
        let e3 = CalendarEvent {
            account_id: account.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60,
            start_time: DateTime::from_timestamp_millis(1000 * 60 * 60).unwrap(),
            user_id: user.id.clone(),
            ..Default::default()
        };
        ctx.repos.events.insert(&e3).await.unwrap();

        let v_e1 = ctx
            .repos
            .reminders
            .init_version(&e1.id)
            .await
            .expect("To create reminder version");
        let v_e2 = ctx
            .repos
            .reminders
            .init_version(&e2.id)
            .await
            .expect("To create reminder version");
        let v_e3 = ctx
            .repos
            .reminders
            .init_version(&e3.id)
            .await
            .expect("To create reminder version");

        let jobs = vec![
            EventRemindersExpansionJob {
                event_id: e1.id.clone(),
                timestamp: DateTime::from_timestamp_millis(1).unwrap(),
                version: v_e1,
            },
            EventRemindersExpansionJob {
                event_id: e2.id.clone(),
                timestamp: DateTime::from_timestamp_millis(2).unwrap(),
                version: v_e2,
            },
            EventRemindersExpansionJob {
                event_id: e3.id.clone(),
                timestamp: DateTime::from_timestamp_millis(3).unwrap(),
                version: v_e3,
            },
        ];
        assert!(
            ctx.repos
                .event_reminders_generation_jobs
                .bulk_insert(&jobs)
                .await
                .map_err(|e| error!("Err: {:?}", e))
                .is_ok()
        );

        // Delete before timestamp
        let delete_res = ctx
            .repos
            .event_reminders_generation_jobs
            .delete_all_before(jobs[1].timestamp)
            .await
            .unwrap();
        assert_eq!(delete_res.len(), 2);
        assert_eq!(delete_res[0], jobs[0]);
        assert_eq!(delete_res[1], jobs[1]);
    }
}
