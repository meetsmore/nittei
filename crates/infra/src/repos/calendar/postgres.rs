use std::convert::{TryFrom, TryInto};

use nittei_domain::{Calendar, ID};
use serde_json::Value;
use sqlx::{
    FromRow,
    PgPool,
    types::{Json, Uuid},
};
use tracing::{error, instrument};

use super::ICalendarRepo;
use crate::repos::shared::query_structs::MetadataFindQuery;

#[derive(Debug)]
pub struct PostgresCalendarRepo {
    pool: PgPool,
}

impl PostgresCalendarRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct CalendarRaw {
    calendar_uid: Uuid,
    user_uid: Uuid,
    account_uid: Uuid,
    name: Option<String>,
    key: Option<String>,
    settings: Value,
    metadata: Value,
}

impl TryFrom<CalendarRaw> for Calendar {
    type Error = anyhow::Error;

    fn try_from(c: CalendarRaw) -> anyhow::Result<Self> {
        Ok(Self {
            id: c.calendar_uid.into(),
            user_id: c.user_uid.into(),
            account_id: c.account_uid.into(),
            name: c.name,
            key: c.key,
            settings: serde_json::from_value(c.settings)?,
            metadata: serde_json::from_value(c.metadata)?,
        })
    }
}

#[async_trait::async_trait]
impl ICalendarRepo for PostgresCalendarRepo {
    #[instrument(name = "calendar::insert")]
    async fn insert(&self, calendar: &Calendar) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO calendars(calendar_uid, account_uid, user_uid, name, key, settings, metadata)
            VALUES($1, $2, $3, $4, $5, $6, $7)
            "#,
            calendar.id.as_ref(),
            calendar.account_id.as_ref(),
            calendar.user_id.as_ref(),
            calendar.name.as_ref(),
            calendar.key.as_ref(),
            Json(&calendar.settings) as _,
            Json(&calendar.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                calendar_id = %calendar.id,
                key = ?calendar.key,
                error = ?e,
                "Failed to insert calendar"
            );
            e
        })?;

        Ok(())
    }

    #[instrument(name = "calendar::save")]
    async fn save(&self, calendar: &Calendar) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE calendars
            SET name = $2,
                key = $3,
                settings = $4,
                metadata = $5
            WHERE calendar_uid = $1
            "#,
            calendar.id.as_ref(),
            calendar.name.as_ref(),
            calendar.key.as_ref(),
            Json(&calendar.settings) as _,
            Json(&calendar.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                calendar_id = %calendar.id,
                key = ?calendar.key,
                error = ?e,
                "Failed to save calendar"
            );
        })?;
        Ok(())
    }

    #[instrument(name = "calendar::find")]
    async fn find(&self, calendar_id: &ID) -> anyhow::Result<Option<Calendar>> {
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            WHERE c.calendar_uid = $1
            "#,
            calendar_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                calendar_id = %calendar_id,
                error = ?e,
                "Failed to find calendar with id"
            );
        })?
        .map(|cal| cal.try_into())
        .transpose()
    }

    #[instrument(name = "calendar::find_multiple")]
    async fn find_multiple(&self, calendar_ids: Vec<&ID>) -> anyhow::Result<Vec<Calendar>> {
        let calendar_ids: Vec<Uuid> = calendar_ids
            .into_iter()
            .map(|id| id.clone().into())
            .collect();
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            WHERE c.calendar_uid = any($1)
            "#,
            calendar_ids.as_slice()
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                calendar_ids = ?calendar_ids,
                error = ?e,
                "Failed to find calendars with ids"
            );
        })?
        .into_iter()
        .map(|c| c.try_into())
        .collect()
    }

    #[instrument(name = "calendar::find_by_user")]
    async fn find_by_user(&self, user_id: &ID) -> anyhow::Result<Vec<Calendar>> {
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            WHERE c.user_uid = $1
            "#,
            user_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                user_id = %user_id,
                error = ?e,
                "Failed to find calendar by user id"
            );
        })?
        .into_iter()
        .map(|c| c.try_into())
        .collect()
    }

    #[instrument(name = "calendar::find_by_user_and_key")]
    async fn find_by_user_and_key(
        &self,
        user_id: &ID,
        key: &str,
    ) -> anyhow::Result<Option<Calendar>> {
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            WHERE c.user_uid = $1 AND c.key = $2
            "#,
            user_id.as_ref(),
            key,
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                user_id = %user_id,
                key = %key,
                error = ?e,
                "Failed to find calendar by user id and key"
            );
        })?
        .map(|cal| cal.try_into())
        .transpose()
    }

    /// Find calendars for multiple users
    #[instrument(name = "calendar::find_for_users")]
    async fn find_for_users(&self, user_ids: &[ID]) -> anyhow::Result<Vec<Calendar>> {
        let user_ids: Vec<Uuid> = user_ids.iter().map(|id| id.clone().into()).collect();
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            WHERE c.user_uid = any($1)
            "#,
            user_ids.as_slice()
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                user_ids = ?user_ids,
                error = ?e,
                "Failed to find calendars by user ids"
            );
        })?
        .into_iter()
        .map(|c| c.try_into())
        .collect()
    }

    #[instrument(name = "calendar::delete")]
    async fn delete(&self, calendar_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM calendars AS c
            WHERE c.calendar_uid = $1
            "#,
            calendar_id.as_ref(),
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .inspect_err(|e| {
            error!(
                calendar_id = %calendar_id,
                error = ?e,
                "Failed to delete calendar with id"
            );
        })?;
        Ok(())
    }

    #[instrument(name = "calendar::find_by_metadata")]
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> anyhow::Result<Vec<Calendar>> {
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            WHERE c.account_uid = $1 AND c.metadata @> $2
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
                query = ?query,
                error = ?e,
                "Failed to find calendars by metadata"
            );
        })?
        .into_iter()
        .map(|c| c.try_into())
        .collect()
    }
}
