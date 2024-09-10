use std::convert::{TryFrom, TryInto};

use nettu_scheduler_domain::{Schedule, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    FromRow,
    PgPool,
};
use tracing::{error, instrument};

use super::IScheduleRepo;
use crate::repos::shared::query_structs::MetadataFindQuery;

#[derive(Debug)]
pub struct PostgresScheduleRepo {
    pool: PgPool,
}

impl PostgresScheduleRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct ScheduleRaw {
    schedule_uid: Uuid,
    user_uid: Uuid,
    account_uid: Uuid,
    rules: Value,
    timezone: String,
    metadata: Value,
}

impl TryFrom<ScheduleRaw> for Schedule {
    type Error = anyhow::Error;
    fn try_from(e: ScheduleRaw) -> anyhow::Result<Self> {
        Ok(Self {
            id: e.schedule_uid.into(),
            user_id: e.user_uid.into(),
            account_id: e.account_uid.into(),
            rules: serde_json::from_value(e.rules).unwrap_or_default(),
            timezone: e.timezone.parse().unwrap_or(chrono_tz::UTC),
            metadata: serde_json::from_value(e.metadata)?,
        })
    }
}

#[async_trait::async_trait]
impl IScheduleRepo for PostgresScheduleRepo {
    #[instrument]
    async fn insert(&self, schedule: &Schedule) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO schedules(schedule_uid, user_uid, rules, timezone, metadata)
            VALUES($1, $2, $3, $4, $5)
            "#,
            schedule.id.as_ref(),
            schedule.user_id.as_ref(),
            Json(&schedule.rules) as _,
            schedule.timezone.to_string(),
            Json(&schedule.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Unable to insert schedule: {:?}. DB returned error: {:?}",
                schedule, e
            );
        })?;

        Ok(())
    }

    #[instrument]
    async fn save(&self, schedule: &Schedule) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE schedules
            SET rules = $2,
            timezone = $3,
            metadata = $4
            WHERE schedule_uid = $1
            "#,
            schedule.id.as_ref(),
            Json(&schedule.rules) as _,
            schedule.timezone.to_string(),
            Json(&schedule.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Unable to save schedule: {:?}. DB returned error: {:?}",
                schedule, e
            );
        })?;
        Ok(())
    }

    #[instrument]
    async fn find(&self, schedule_id: &ID) -> anyhow::Result<Option<Schedule>> {
        sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT s.*, u.account_uid FROM schedules AS s
            INNER JOIN users AS u
                ON u.user_uid = s.user_uid
            WHERE s.schedule_uid = $1
            "#,
            schedule_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find schedule with id: {:?} failed. DB returned error: {:?}",
                schedule_id, e
            );
        })?
        .map(|schedule| schedule.try_into())
        .transpose()
    }

    #[instrument]
    async fn find_many(&self, schedule_ids: &[ID]) -> anyhow::Result<Vec<Schedule>> {
        let ids = schedule_ids
            .iter()
            .map(|id| *id.as_ref())
            .collect::<Vec<_>>();
        sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT s.*, u.account_uid FROM schedules AS s
            INNER JOIN users AS u
                ON u.user_uid = s.user_uid
            WHERE s.schedule_uid = ANY($1)
            "#,
            &ids
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find schedules with ids: {:?} failed. DB returned error: {:?}",
                schedule_ids, e
            );
        })?
        .into_iter()
        .map(|e| e.try_into())
        .collect()
    }

    #[instrument]
    async fn find_by_user(&self, user_id: &ID) -> anyhow::Result<Vec<Schedule>> {
        sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT s.*, u.account_uid FROM schedules AS s
            INNER JOIN users AS u
                ON u.user_uid = s.user_uid
            WHERE s.user_uid = $1
            "#,
            user_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find schedules for user id: {:?} failed. DB returned error: {:?}",
                user_id, e
            );
        })?
        .into_iter()
        .map(|s| s.try_into())
        .collect()
    }

    #[instrument]
    async fn delete(&self, schedule_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM schedules AS s
            WHERE s.schedule_uid = $1
            RETURNING *
            "#,
            schedule_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Delete schedule with id: {:?} failed. DB returned error: {:?}",
                schedule_id, e
            );
        })?
        .ok_or_else(|| anyhow::Error::msg("Unable to delete schedule"))
        .map(|_| ())
    }

    #[instrument]
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> anyhow::Result<Vec<Schedule>> {
        sqlx::query_as!(
            ScheduleRaw,
            r#"
            SELECT s.*, u.account_uid FROM schedules AS s
            INNER JOIN users AS u
                ON u.user_uid = s.user_uid
            WHERE u.account_uid = $1 AND s.metadata @> $2
            LIMIT $3
            OFFSET $4
            "#,
            query.account_id.as_ref(),
            Json(&query.metadata) as _,
            query.limit as i64,
            query.skip as i64,
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find schedules by metadata: {:?} failed. DB returned error: {:?}",
                query, e
            );
        })?
        .into_iter()
        .map(|s| s.try_into())
        .collect()
    }
}
