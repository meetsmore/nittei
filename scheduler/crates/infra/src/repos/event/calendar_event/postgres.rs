use chrono::{DateTime, Utc};
use nettu_scheduler_domain::{CalendarEvent, CalendarEventReminder, RRuleOptions, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    FromRow,
    PgPool,
};
use tracing::{error, instrument};

use super::{IEventRepo, MostRecentCreatedServiceEvents};
use crate::repos::shared::query_structs::MetadataFindQuery;

#[derive(Debug)]
pub struct PostgresEventRepo {
    pool: PgPool,
}

impl PostgresEventRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct MostRecentCreatedServiceEventsRaw {
    user_uid: Uuid,
    created: Option<i64>,
}

impl From<MostRecentCreatedServiceEventsRaw> for MostRecentCreatedServiceEvents {
    fn from(e: MostRecentCreatedServiceEventsRaw) -> Self {
        Self {
            user_id: e.user_uid.into(),
            created: e.created,
        }
    }
}

#[derive(Debug, FromRow)]
struct EventRaw {
    event_uid: Uuid,
    calendar_uid: Uuid,
    user_uid: Uuid,
    account_uid: Uuid,
    start_time: DateTime<Utc>,
    duration: i64,
    busy: bool,
    end_time: DateTime<Utc>,
    created: i64,
    updated: i64,
    recurrence: Option<Value>,
    exdates: Vec<DateTime<Utc>>,
    reminders: Option<Value>,
    service_uid: Option<Uuid>,
    metadata: Value,
}

impl From<EventRaw> for CalendarEvent {
    fn from(e: EventRaw) -> Self {
        let recurrence: Option<RRuleOptions> = match e.recurrence {
            Some(json) => serde_json::from_value(json).unwrap(),
            None => None,
        };
        let reminders: Vec<CalendarEventReminder> = match e.reminders {
            Some(json) => serde_json::from_value(json).unwrap(),
            None => Vec::new(),
        };

        Self {
            id: e.event_uid.into(),
            user_id: e.user_uid.into(),
            account_id: e.account_uid.into(),
            calendar_id: e.calendar_uid.into(),
            start_time: e.start_time,
            duration: e.duration,
            busy: e.busy,
            end_time: e.end_time,
            created: e.created,
            updated: e.updated,
            recurrence,
            exdates: e.exdates,
            reminders,
            service_id: e.service_uid.map(|id| id.into()),
            metadata: serde_json::from_value(e.metadata).unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl IEventRepo for PostgresEventRepo {
    #[instrument]
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO calendar_events(
                event_uid,
                calendar_uid,
                start_time,
                duration,
                end_time,
                busy,
                created,
                updated,
                recurrence,
                exdates,
                reminders,
                service_uid,
                metadata
            )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            e.id.as_ref(),
            e.calendar_id.as_ref(),
            e.start_time,
            e.duration,
            e.end_time,
            e.busy,
            e.created,
            e.updated,
            Json(&e.recurrence) as _,
            &e.exdates,
            Json(&e.reminders) as _,
            e.service_id.as_ref().map(|id| id.as_ref()),
            Json(&e.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            error!(
                "Unable to insert calendar_event: {:?}. DB returned error: {:?}",
                e, err
            );
            err
        })?;

        Ok(())
    }

    #[instrument]
    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE calendar_events SET
                start_time = $2,
                duration = $3,
                end_time = $4,
                busy = $5,
                created = $6,
                updated = $7,
                recurrence = $8,
                exdates = $9,
                reminders = $10,
                service_uid = $11,
                metadata = $12
            WHERE event_uid = $1
            "#,
            e.id.as_ref(),
            e.start_time,
            e.duration,
            e.end_time,
            e.busy,
            e.created,
            e.updated,
            Json(&e.recurrence) as _,
            &e.exdates,
            Json(&e.reminders) as _,
            e.service_id.as_ref().map(|id| id.as_ref()),
            Json(&e.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            error!(
                "Unable to save calendar_event: {:?}. DB returned error: {:?}",
                e, err
            );
            err
        })?;

        Ok(())
    }

    #[instrument]
    async fn find(&self, event_id: &ID) -> Option<CalendarEvent> {
        let res: Option<EventRaw> = sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.event_uid = $1
            "#,
            event_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| {
            error!(
                "Find calendar event with id: {:?} failed. DB returned error: {:?}",
                event_id, err
            );

            err
        })
        .ok()?;

        res.map(|e| e.into())
    }

    #[instrument]
    async fn find_many(&self, event_ids: &[ID]) -> anyhow::Result<Vec<CalendarEvent>> {
        let ids = event_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();
        let events: Vec<EventRaw> = sqlx::query_as(
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.event_uid = ANY($1)
            "#,
        )
        .bind(&ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Find calendar events with ids: {:?} failed. DB returned error: {:?}",
                event_ids, e
            );
            e
        })?;
        Ok(events.into_iter().map(|e| e.into()).collect())
    }

    #[instrument]
    async fn find_by_calendar(
        &self,
        calendar_id: &ID,
        timespan: Option<&nettu_scheduler_domain::TimeSpan>,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let events: Vec<EventRaw> = match timespan {
            Some(timespan) => sqlx::query_as!(
                EventRaw,
                r#"
                    SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
                    INNER JOIN calendars AS c
                        ON c.calendar_uid = e.calendar_uid
                    INNER JOIN users AS u
                        ON u.user_uid = c.user_uid
                    WHERE e.calendar_uid = $1 AND
                    e.start_time <= $2 AND e.end_time >= $3
                    "#,
                calendar_id.as_ref(),
                timespan.end(),
                timespan.start()
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "Find calendar events for calendar id: {:?} failed. DB returned error: {:?}",
                    calendar_id, e
                );
                e
            })
            .unwrap_or_default(),
            None => sqlx::query_as!(
                EventRaw,
                r#"
                    SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
                    INNER JOIN calendars AS c
                        ON c.calendar_uid = e.calendar_uid
                    INNER JOIN users AS u
                        ON u.user_uid = c.user_uid
                    WHERE e.calendar_uid = $1
                    "#,
                calendar_id.as_ref(),
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "Find calendar events for calendar id: {:?} failed. DB returned error: {:?}",
                    calendar_id, e
                );
                e
            })
            .unwrap_or_default(),
        };
        Ok(events.into_iter().map(|e| e.into()).collect())
    }

    #[instrument]
    async fn find_by_calendars(
        &self,
        calendar_ids: Vec<ID>,
        timespan: &nettu_scheduler_domain::TimeSpan,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let calendar_ids: Vec<Uuid> = calendar_ids
            .into_iter()
            .map(|id| id.clone().into())
            .collect();
        let events: Vec<EventRaw> = sqlx::query_as!(
            EventRaw,
            r#"
                    SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
                    INNER JOIN calendars AS c
                        ON c.calendar_uid = e.calendar_uid
                    INNER JOIN users AS u
                        ON u.user_uid = c.user_uid
                    WHERE e.calendar_uid  = any($1) AND
                    e.start_time <= $2 AND e.end_time >= $3
                    "#,
            calendar_ids.as_slice(),
            timespan.end(),
            timespan.start()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Find calendar events for calendar ids: {:?} failed. DB returned error: {:?}",
                calendar_ids, e
            );
            e
        })
        .unwrap_or_default();
        Ok(events.into_iter().map(|e| e.into()).collect())
    }

    #[instrument]
    async fn find_most_recently_created_service_events(
        &self,
        service_id: &ID,
        user_ids: &[ID],
    ) -> Vec<MostRecentCreatedServiceEvents> {
        let user_ids = user_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();
        // https://github.com/launchbadge/sqlx/issues/367
        let events: Vec<MostRecentCreatedServiceEventsRaw> = match sqlx::query_as(
            r#"
            SELECT users.user_uid, events.created FROM users LEFT JOIN (
                SELECT DISTINCT ON (user_uid) user_uid, e.created
                FROM calendar_events AS e
                INNER JOIN calendars AS c
                    ON c.calendar_uid = e.calendar_uid
                WHERE service_uid = $1
                ORDER BY user_uid, created DESC
            ) AS events ON events.user_uid = users.user_uid
            WHERE users.user_uid = ANY($2)
            "#,
        )
        .bind(service_id.as_ref())
        .bind(&user_ids)
        .fetch_all(&self.pool)
        .await
        {
            Ok(events) => events,
            Err(e) => {
                error!(
                    "Find most recently created service events for service id: {} failed. DB returned error: {:?}",
                    service_id, e
                );
                return Vec::new();
            }
        };
        events.into_iter().map(|e| e.into()).collect()
    }

    #[instrument]
    async fn find_by_service(
        &self,
        service_id: &ID,
        user_ids: &[ID],
        min_time: DateTime<Utc>,
        max_time: DateTime<Utc>,
    ) -> Vec<CalendarEvent> {
        let user_ids = user_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();
        let events: Vec<EventRaw> = match sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.service_uid = $1 AND
            u.user_uid = ANY($2) AND
            e.start_time <= $3 AND e.end_time >= $4
            "#,
            service_id.as_ref(),
            &user_ids,
            max_time,
            min_time,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(events) => events,
            Err(e) => {
                error!(
                    "Find calendar events for service id: {}, user_ids: {:?}, min_time: {}, max_time: {} failed. DB returned error: {:?}",
                    service_id,
                    user_ids,
                    min_time,
                    max_time,
                     e
                );

                return Vec::new();
            }
        };
        events.into_iter().map(|e| e.into()).collect()
    }

    #[instrument]
    async fn find_user_service_events(
        &self,
        user_id: &ID,
        busy: bool,
        min_time: DateTime<Utc>,
        max_time: DateTime<Utc>,
    ) -> Vec<CalendarEvent> {
        let events: Vec<EventRaw> = match sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.user_uid = $1 AND
            e.busy = $2 AND
            e.service_uid IS NOT NULL AND
            e.start_time <= $3 AND e.end_time >= $4
            "#,
            user_id.as_ref(),
            busy,
            max_time,
            min_time,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(events) => events,
            Err(e) => {
                error!(
                    "Find service calendar events for user_id: {}, busy: {}, min_time: {}, max_time: {} failed. DB returned error: {:?}",
                    user_id,
                    busy,
                    min_time,
                    max_time,
                     e
                );

                return Vec::new();
            }
        };
        events.into_iter().map(|e| e.into()).collect()
    }

    #[instrument]
    async fn delete(&self, event_id: &ID) -> anyhow::Result<()> {
        let res = sqlx::query!(
            r#"
            DELETE FROM calendar_events AS c
            WHERE c.event_uid = $1
            RETURNING *
            "#,
            event_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Delete calendar event with id: {:?} failed. DB returned error: {:?}",
                event_id, e
            );
            e
        })?;

        if res.is_some() {
            Ok(())
        } else {
            Err(anyhow::Error::msg("Unable to delete calendar event"))
        }
    }

    #[instrument]
    async fn delete_by_service(&self, service_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM calendar_events AS c
            WHERE c.service_uid = $1
            "#,
            service_id.as_ref(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Delete calendar event by service id: {:?} failed. DB returned error: {:?}",
                service_id, e
            );
            e
        })?;
        Ok(())
    }

    #[instrument]
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<CalendarEvent> {
        let events: Vec<EventRaw> = sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.account_uid = $1 AND e.metadata @> $2
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
                "Find calendar events by metadata: {:?} failed. DB returned error: {:?}",
                query, e
            );
            e
        })
        .unwrap_or_default();

        events.into_iter().map(|e| e.into()).collect()
    }
}
