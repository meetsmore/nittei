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

    fn try_from(e: CalendarRaw) -> anyhow::Result<Self> {
        Ok(Self {
            id: e.calendar_uid.into(),
            user_id: e.user_uid.into(),
            account_id: e.account_uid.into(),
            name: e.name,
            key: e.key,
            settings: serde_json::from_value(e.settings)?,
            metadata: serde_json::from_value(e.metadata)?,
        })
    }
}

#[async_trait::async_trait]
impl ICalendarRepo for PostgresCalendarRepo {
    #[instrument]
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
                "Unable to insert calendar: {:?}. DB returned error: {:?}",
                calendar, e
            );
            e
        })?;

        Ok(())
    }

    #[instrument]
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
                "Unable to save calendar: {:?}. DB returned error: {:?}",
                calendar, e
            );
        })?;
        Ok(())
    }

    #[instrument]
    async fn find(&self, calendar_id: &ID) -> anyhow::Result<Option<Calendar>> {
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE c.calendar_uid = $1
            "#,
            calendar_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find calendar with id: {:?} failed. DB returned error: {:?}",
                calendar_id, e
            );
        })?
        .map(|cal| cal.try_into())
        .transpose()
    }

    #[instrument]
    async fn find_multiple(&self, calendar_ids: Vec<&ID>) -> anyhow::Result<Vec<Calendar>> {
        let calendar_ids: Vec<Uuid> = calendar_ids
            .into_iter()
            .map(|id| id.clone().into())
            .collect();
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE c.calendar_uid = any($1)
            "#,
            calendar_ids.as_slice()
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find calendars with ids: {:?} failed. DB returned error: {:?}",
                calendar_ids, e
            );
        })?
        .into_iter()
        .map(|c| c.try_into())
        .collect()
    }

    #[instrument]
    async fn find_by_user(&self, user_id: &ID) -> anyhow::Result<Vec<Calendar>> {
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE c.user_uid = $1
            "#,
            user_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find calendar by user id: {:?} failed. DB returned error: {:?}",
                user_id, e
            );
        })?
        .into_iter()
        .map(|c| c.try_into())
        .collect()
    }

    async fn find_by_user_and_key(
        &self,
        user_id: &ID,
        key: &str,
    ) -> anyhow::Result<Option<Calendar>> {
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE c.user_uid = $1 AND c.key = $2
            "#,
            user_id.as_ref(),
            key,
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find calendar by user id: {:?} and key: {:?} failed. DB returned error: {:?}",
                user_id, key, e
            );
        })?
        .map(|cal| cal.try_into())
        .transpose()
    }

    #[instrument]
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
                "Delete calendar with id: {:?} failed. DB returned error: {:?}",
                calendar_id, e
            );
        })?;
        Ok(())
    }

    #[instrument]
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> anyhow::Result<Vec<Calendar>> {
        sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.* FROM calendars AS c
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.account_uid = $1 AND c.metadata @> $2
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
                "Find calendars by metadata: {:?} failed. DB returned error: {:?}",
                query, e
            );
        })?
        .into_iter()
        .map(|c| c.try_into())
        .collect()
    }
}
