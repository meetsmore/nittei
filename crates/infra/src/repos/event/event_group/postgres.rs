use nittei_domain::{event_group::EventGroup, ID};
use sqlx::{FromRow, PgPool};
use tracing::{error, instrument};

use super::IEventGroupRepo;

#[derive(Debug)]
pub struct PostgresEventGroupRepo {
    pool: PgPool,
}

impl PostgresEventGroupRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct EventGroupRaw {
    pub group_uid: ID,
    pub calendar_uid: ID,
    pub user_uid: ID,
    pub account_uid: ID,
    pub parent_id: Option<String>,
    pub external_id: Option<String>,
}

impl From<EventGroupRaw> for EventGroup {
    fn from(e: EventGroupRaw) -> Self {
        Self {
            id: e.group_uid,
            calendar_id: e.calendar_uid,
            user_id: e.user_uid,
            account_id: e.account_uid,
            parent_id: e.parent_id,
            external_id: e.external_id,
        }
    }
}

#[async_trait::async_trait]
impl IEventGroupRepo for PostgresEventGroupRepo {
    #[instrument]
    async fn insert(&self, e: &EventGroup) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO events_groups(
                group_uid,
                calendar_uid,
                parent_id,
                external_id
            )
            VALUES($1, $2, $3, $4)
            "#,
            e.id.as_ref(),
            e.calendar_id.as_ref(),
            e.parent_id,
            e.external_id,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Unable to insert event_group: {:?}. DB returned error: {:?}",
                e, err
            );
        })?;

        Ok(())
    }

    #[instrument]
    async fn save(&self, e: &EventGroup) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE events_groups SET
                parent_id = $2,
                external_id = $3
            WHERE group_uid = $1
            "#,
            e.id.as_ref(),
            e.parent_id,
            e.external_id,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Unable to save event_group: {:?}. DB returned error: {:?}",
                e, err
            );
        })?;

        Ok(())
    }

    #[instrument]
    async fn find(&self, group_id: &ID) -> anyhow::Result<Option<EventGroup>> {
        Ok(sqlx::query_as!(
            EventGroupRaw,
            r#"
            SELECT g.*, u.user_uid, account_uid FROM events_groups AS g
            INNER JOIN calendars AS c
                ON c.calendar_uid = g.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE g.group_uid = $1
            "#,
            group_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Find event_group with id: {:?} failed. DB returned error: {:?}",
                group_id, err
            );
        })?
        .map(|e| e.into()))
    }

    #[instrument]
    async fn get_by_external_id(&self, external_id: &str) -> anyhow::Result<Option<EventGroup>> {
        Ok(sqlx::query_as!(
            EventGroupRaw,
            r#"
            SELECT g.*, u.user_uid, account_uid FROM events_groups AS g
            INNER JOIN calendars AS c
                ON c.calendar_uid = g.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE g.external_id = $1
            "#,
            external_id,
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Find events_groups with external_id: {:?} failed. DB returned error: {:?}",
                external_id, err
            );
        })?
        .map(|e| e.into()))
    }

    #[instrument]
    async fn delete(&self, group_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM events_groups AS g
            WHERE g.group_uid = $1
            RETURNING *
            "#,
            group_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Delete events_groups with id: {:?} failed. DB returned error: {:?}",
                group_id, e
            );
        })?
        .ok_or_else(|| anyhow::Error::msg("Unable to delete event_group"))
        .map(|_| ())
    }
}
