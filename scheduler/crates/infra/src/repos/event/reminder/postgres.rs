use chrono::{DateTime, Utc};
use nettu_scheduler_domain::{Reminder, ID};
use sqlx::{types::Uuid, FromRow, PgPool};
use tracing::{error, instrument};

use super::IReminderRepo;

#[derive(Debug)]
pub struct PostgresReminderRepo {
    pool: PgPool,
}

impl PostgresReminderRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct ReminderVersionRaw {
    #[allow(dead_code)]
    event_uid: Uuid,
    version: i64,
}

#[derive(Debug, FromRow)]
struct ReminderRaw {
    event_uid: Uuid,
    account_uid: Uuid,
    remind_at: DateTime<Utc>,
    version: i64,
    identifier: String,
}

impl From<ReminderRaw> for Reminder {
    fn from(e: ReminderRaw) -> Self {
        Self {
            event_id: e.event_uid.into(),
            account_id: e.account_uid.into(),
            remind_at: e.remind_at,
            version: e.version,
            identifier: e.identifier,
        }
    }
}

#[async_trait::async_trait]
impl IReminderRepo for PostgresReminderRepo {
    #[instrument]
    async fn bulk_insert(&self, reminders: &[Reminder]) -> anyhow::Result<()> {
        for reminder in reminders {
            sqlx::query!(
                r#"
            INSERT INTO reminders
            (event_uid, account_uid, remind_at, version, identifier)
            VALUES($1, $2, $3, $4, $5)
            "#,
                reminder.event_id.as_ref(),
                reminder.account_id.as_ref(),
                reminder.remind_at,
                reminder.version as _,
                reminder.identifier,
            )
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "Unable to insert calendar event reminder: {:?}. DB returned error: {:?}",
                    reminder, e
                );
                e
            })?;
        }
        Ok(())
    }

    #[instrument]
    async fn init_version(&self, event_id: &ID) -> anyhow::Result<i64> {
        let r_version = sqlx::query_as!(
            ReminderVersionRaw,
            r#"
            INSERT INTO event_reminder_versions
                (event_uid)
            VALUES
                ($1)
            RETURNING *
            "#,
            event_id.as_ref(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            error!(
                "Unable to insert calendar event reminder version for event id: {}. DB returned error: {:?}",
                event_id, err
            );
            err
        })?;

        Ok(r_version.version)
    }

    #[instrument]
    async fn inc_version(&self, event_id: &ID) -> anyhow::Result<i64> {
        let r_version = sqlx::query_as!(
            ReminderVersionRaw,
            r#"
            WITH prev_v AS (
                DELETE FROM event_reminder_versions
                WHERE event_uid = $1
                RETURNING *
            )
            INSERT INTO event_reminder_versions
                (event_uid, version)
            SELECT $1, version + 1 from prev_v
            RETURNING *
            "#,
            event_id.as_ref(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            error!(
                "Unable to increment calendar event reminder version for event id: {}. DB returned error: {:?}",
                event_id, err
            );
            err
        })?;

        Ok(r_version.version)
    }

    #[instrument]
    async fn delete_all_before(&self, before: DateTime<Utc>) -> Vec<Reminder> {
        sqlx::query_as!(
            ReminderRaw,
            r#"
            DELETE FROM reminders AS r
            WHERE r.remind_at <= $1
            RETURNING *
            "#,
            before,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Unable to delete calendar event reminders before timestamp: {}. DB returned error: {:?}",
                before, e
            );
            e
        })
        .unwrap_or_default()
        .into_iter()
        .map(|reminder| reminder.into())
        .collect()
    }
}
