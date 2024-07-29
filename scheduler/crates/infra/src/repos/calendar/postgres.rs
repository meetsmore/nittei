use nettu_scheduler_domain::{Calendar, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    FromRow,
    PgPool,
};
use tracing::error;

use super::ICalendarRepo;
use crate::repos::shared::query_structs::MetadataFindQuery;

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
    settings: Value,
    metadata: Value,
}

impl From<CalendarRaw> for Calendar {
    fn from(e: CalendarRaw) -> Self {
        Self {
            id: e.calendar_uid.into(),
            user_id: e.user_uid.into(),
            account_id: e.account_uid.into(),
            settings: serde_json::from_value(e.settings).unwrap(),
            metadata: serde_json::from_value(e.metadata).unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl ICalendarRepo for PostgresCalendarRepo {
    async fn insert(&self, calendar: &Calendar) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO calendars(calendar_uid, user_uid, settings, metadata)
            VALUES($1, $2, $3, $4)
            "#,
            calendar.id.as_ref(),
            calendar.user_id.as_ref(),
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

    async fn save(&self, calendar: &Calendar) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE calendars
            SET settings = $2,
            metadata = $3
            WHERE calendar_uid = $1
            "#,
            calendar.id.as_ref(),
            Json(&calendar.settings) as _,
            Json(&calendar.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Unable to save calendar: {:?}. DB returned error: {:?}",
                calendar, e
            );
            e
        })?;
        Ok(())
    }

    async fn find(&self, calendar_id: &ID) -> Option<Calendar> {
        let res: Option<CalendarRaw> = sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.*, u.account_uid FROM calendars AS c
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE c.calendar_uid = $1
            "#,
            calendar_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Find calendar with id: {:?} failed. DB returned error: {:?}",
                calendar_id, e
            );
            e
        })
        .ok()?;

        res.map(|cal| cal.into())
    }

    async fn find_by_user(&self, user_id: &ID) -> Vec<Calendar> {
        let calendars: Vec<CalendarRaw> = sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.*, u.account_uid FROM calendars AS c
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE c.user_uid = $1
            "#,
            user_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Find calendar by user id: {:?} failed. DB returned error: {:?}",
                user_id, e
            );
            e
        })
        .unwrap_or_default();

        calendars.into_iter().map(|c| c.into()).collect()
    }

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
        .map_err(|e| {
            error!(
                "Delete calendar with id: {:?} failed. DB returned error: {:?}",
                calendar_id, e
            );

            anyhow::Error::new(e)
        })
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Calendar> {
        let calendars: Vec<CalendarRaw> = sqlx::query_as!(
            CalendarRaw,
            r#"
            SELECT c.*, u.account_uid FROM calendars AS c
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
        .map_err(|e| {
            error!(
                "Find calendars by metadata: {:?} failed. DB returned error: {:?}",
                query, e
            );
            e
        })
        .unwrap_or_default();

        calendars.into_iter().map(|c| c.into()).collect()
    }
}
