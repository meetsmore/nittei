use nittei_domain::{ID, SyncedCalendarEvent};
use sqlx::{FromRow, PgPool, types::Uuid};
use tracing::{error, instrument};

use super::IEventSyncedRepo;

#[derive(Debug)]
pub struct PostgresEventSyncedRepo {
    pool: PgPool,
}

impl PostgresEventSyncedRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct SyncedEventRaw {
    event_uid: Uuid,
    calendar_uid: Uuid,
    user_uid: Uuid,
    ext_calendar_id: String,
    ext_calendar_event_id: String,
    provider: String,
}

impl From<SyncedEventRaw> for SyncedCalendarEvent {
    fn from(e: SyncedEventRaw) -> Self {
        Self {
            event_id: e.event_uid.into(),
            user_id: e.user_uid.into(),
            calendar_id: e.calendar_uid.into(),
            ext_calendar_id: e.ext_calendar_id,
            ext_event_id: e.ext_calendar_event_id,
            provider: e.provider.into(),
        }
    }
}

#[async_trait::async_trait]
impl IEventSyncedRepo for PostgresEventSyncedRepo {
    #[instrument]
    async fn insert(&self, e: &SyncedCalendarEvent) -> anyhow::Result<()> {
        let provider: String = e.provider.clone().into();
        sqlx::query!(
            r#"
            INSERT INTO externally_synced_calendar_events(
                event_uid,
                calendar_uid,
                ext_calendar_id,
                ext_calendar_event_id,
                provider
            )
            VALUES($1, $2, $3, $4, $5)
            "#,
            e.event_id.as_ref(),
            e.calendar_id.as_ref(),
            e.ext_calendar_id,
            e.ext_event_id,
            provider as _
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            error!(
                "Unable to insert syunced calendar event: {:?}. DB returned error: {:?}",
                e, err
            );
            err
        })?;

        Ok(())
    }

    #[instrument]
    async fn find_by_event(&self, event_id: &ID) -> anyhow::Result<Vec<SyncedCalendarEvent>> {
        let synced_events: Vec<SyncedEventRaw> = sqlx::query_as!(
            SyncedEventRaw,
            r#"
            SELECT e.*, c.user_uid FROM externally_synced_calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            WHERE e.event_uid = $1
            "#,
            event_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Unable to find synced calendar events for calendar event with id: {}. DB returned error: {:?}",
                event_id, e
            );
            e
        })
        ?;

        Ok(synced_events.into_iter().map(|e| e.into()).collect())
    }
}
