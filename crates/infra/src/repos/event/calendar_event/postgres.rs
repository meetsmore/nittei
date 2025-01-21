use std::convert::{TryFrom, TryInto};

use chrono::{DateTime, Utc};
use nittei_domain::{CalendarEvent, CalendarEventReminder, RRuleOptions, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    FromRow,
    PgPool,
    QueryBuilder,
};
use tracing::{error, instrument};

use super::{
    IEventRepo,
    MostRecentCreatedServiceEvents,
    SearchEventsForAccountParams,
    SearchEventsForUserParams,
};
use crate::repos::{
    apply_datetime_query,
    apply_id_query,
    apply_string_query,
    shared::query_structs::MetadataFindQuery,
};

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

#[derive(Debug, FromRow, Clone)]
struct EventRaw {
    event_uid: Uuid,
    calendar_uid: Uuid,
    user_uid: Uuid,
    account_uid: Uuid,
    parent_id: Option<String>,
    external_id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    event_type: Option<String>,
    location: Option<String>,
    all_day: bool,
    status: String,
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
    group_uid: Option<Uuid>,
    metadata: Value,
}

impl TryFrom<EventRaw> for CalendarEvent {
    type Error = anyhow::Error;

    fn try_from(e: EventRaw) -> anyhow::Result<Self> {
        let recurrence: Option<RRuleOptions> = match e.recurrence {
            Some(json) => serde_json::from_value(json)?,
            None => None,
        };
        let reminders: Vec<CalendarEventReminder> = match e.reminders {
            Some(json) => serde_json::from_value(json)?,
            None => Vec::new(),
        };

        Ok(Self {
            id: e.event_uid.into(),
            user_id: e.user_uid.into(),
            account_id: e.account_uid.into(),
            calendar_id: e.calendar_uid.into(),
            parent_id: e.parent_id,
            external_id: e.external_id,
            title: e.title,
            description: e.description,
            event_type: e.event_type,
            location: e.location,
            all_day: e.all_day,
            status: e.status.try_into()?,
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
            group_id: e.group_uid.map(|id| id.into()),
            metadata: serde_json::from_value(e.metadata)?,
        })
    }
}

#[async_trait::async_trait]
impl IEventRepo for PostgresEventRepo {
    #[instrument]
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        let status: String = e.status.clone().into();
        sqlx::query!(
            r#"
            INSERT INTO calendar_events(
                event_uid,
                calendar_uid,
                parent_id,
                external_id,
                title,
                description,
                event_type,
                location,
                status,
                all_day,
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
                group_uid,
                metadata
            )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            "#,
            e.id.as_ref(),
            e.calendar_id.as_ref(),
            e.parent_id,
            e.external_id,
            e.title,
            e.description,
            e.event_type,
            e.location,
            status,
            e.all_day,
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
            e.group_id.as_ref().map(|id| id.as_ref()),
            Json(&e.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Unable to insert calendar_event: {:?}. DB returned error: {:?}",
                e, err
            );
        })?;

        Ok(())
    }

    #[instrument]
    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        let status: String = e.status.clone().into();
        sqlx::query!(
            r#"
            UPDATE calendar_events SET
                parent_id = $2,
                external_id = $3,
                title = $4,
                description = $5,
                event_type = $6,
                location = $7,
                status = $8,
                all_day = $9,
                start_time = $10,
                duration = $11,
                end_time = $12,
                busy = $13,
                created = $14,
                updated = $15,
                recurrence = $16,
                exdates = $17,
                reminders = $18,
                service_uid = $19,
                group_uid = $20,
                metadata = $21
            WHERE event_uid = $1
            "#,
            e.id.as_ref(),
            e.parent_id,
            e.external_id,
            e.title,
            e.description,
            e.event_type,
            e.location,
            status,
            e.all_day,
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
            e.group_id.as_ref().map(|id| id.as_ref()),
            Json(&e.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Unable to save calendar_event: {:?}. DB returned error: {:?}",
                e, err
            );
        })?;

        Ok(())
    }

    #[instrument]
    async fn find(&self, event_id: &ID) -> anyhow::Result<Option<CalendarEvent>> {
        sqlx::query_as!(
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
        .inspect_err(|err| {
            error!(
                "Find calendar event with id: {:?} failed. DB returned error: {:?}",
                event_id, err
            );
        })?
        .map(|e| e.try_into())
        .transpose()
    }

    #[instrument]
    async fn get_by_external_id(&self, external_id: &str) -> anyhow::Result<Option<CalendarEvent>> {
        sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.external_id = $1
            "#,
            external_id,
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Find calendar event with external_id: {:?} failed. DB returned error: {:?}",
                external_id, err
            );
        })?
        .map(|e| e.try_into())
        .transpose()
    }

    #[instrument]
    async fn find_many(&self, event_ids: &[ID]) -> anyhow::Result<Vec<CalendarEvent>> {
        let ids = event_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();
        sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid, account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.event_uid = ANY($1)
            "#,
            &ids
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find calendar events with ids: {:?} failed. DB returned error: {:?}",
                event_ids, e
            );
        })?
        .into_iter()
        .map(|e| e.try_into())
        .collect()
    }

    #[instrument]
    async fn find_by_calendar(
        &self,
        calendar_id: &ID,
        timespan: Option<&nittei_domain::TimeSpan>,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        if let Some(timespan) = timespan {
            sqlx::query_as!(
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
            .inspect_err(|e| {
                error!(
                    "Find calendar events for calendar id: {:?} failed. DB returned error: {:?}",
                    calendar_id, e
                );
            })?
            .into_iter()
            .map(|e| e.try_into())
            .collect()
        } else {
            sqlx::query_as!(
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
            .inspect_err(|e| {
                error!(
                    "Find calendar events for calendar id: {:?} failed. DB returned error: {:?}",
                    calendar_id, e
                );
            })?
            .into_iter()
            .map(|e| e.try_into())
            .collect()
        }
    }

    #[instrument]
    async fn find_by_calendars(
        &self,
        calendar_ids: Vec<ID>,
        timespan: &nittei_domain::TimeSpan,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let calendar_ids: Vec<Uuid> = calendar_ids
            .into_iter()
            .map(|id| id.clone().into())
            .collect();
        sqlx::query_as!(
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
        .inspect_err(|e| {
            error!(
                "Find calendar events for calendar ids: {:?} failed. DB returned error: {:?}",
                calendar_ids, e
            );
        })?
        .into_iter()
        .map(|e| e.try_into())
        .collect()
    }

    /// Search events
    /// This method is used to search events based on the given parameters for one specific user:
    /// The parameters are optional and can be used to filter the events
    ///
    /// Warning: performance of this method might not optimal
    async fn search_events_for_user(
        &self,
        params: SearchEventsForUserParams,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT e.*, c.user_uid, u.account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.user_uid = "#,
        );

        query.push_bind::<Uuid>(params.user_id.into());

        if let Some(calendar_ids) = params.calendar_ids {
            query.push(" AND c.calendar_uid IN (");
            let mut separated = query.separated(", ");
            for value_type in calendar_ids.iter() {
                separated.push_bind::<Uuid>(value_type.clone().into());
            }
            separated.push_unseparated(") ");
        }

        apply_string_query(
            &mut query,
            "parent_id",
            &params.search_events_params.parent_id,
        );

        apply_id_query(
            &mut query,
            "group_uuid",
            &params.search_events_params.group_id,
        );

        apply_datetime_query(
            &mut query,
            "start_time",
            &params.search_events_params.start_time,
        );

        apply_datetime_query(
            &mut query,
            "end_time",
            &params.search_events_params.end_time,
        );

        apply_string_query(
            &mut query,
            "event_type",
            &params.search_events_params.event_type,
        );

        apply_string_query(&mut query, "status", &params.search_events_params.status);

        apply_datetime_query(
            &mut query,
            "updated_at",
            &params.search_events_params.updated_at,
        );

        if let Some(metadata) = params.search_events_params.metadata {
            query.push(" AND e.metadata @> ");
            query.push_bind(Json(metadata.clone()));
        }

        let rows = query.build().fetch_all(&self.pool).await.inspect_err(|e| {
            error!("Search events failed. DB returned error: {:?}", e);
        })?;

        let events_raw: Vec<EventRaw> = rows
            .into_iter()
            .map(|row| EventRaw::from_row(&row))
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        events_raw
            .into_iter()
            .map(CalendarEvent::try_from)
            .collect()
    }

    /// Search events at the account level
    /// This method is used to search events for all users of an account based on the given parameters
    /// The parameters are optional and can be used to filter the events
    ///
    /// Warning: performance of this method might not optimal
    async fn search_events_for_account(
        &self,
        params: SearchEventsForAccountParams,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT e.*, c.user_uid, u.account_uid FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.account_uid = "#,
        );

        query.push_bind::<Uuid>(params.account_id.into());

        apply_string_query(
            &mut query,
            "parent_id",
            &params.search_events_params.parent_id,
        );

        apply_id_query(
            &mut query,
            "group_uuid",
            &params.search_events_params.group_id,
        );

        apply_datetime_query(
            &mut query,
            "start_time",
            &params.search_events_params.start_time,
        );

        apply_datetime_query(
            &mut query,
            "end_time",
            &params.search_events_params.end_time,
        );

        apply_string_query(
            &mut query,
            "event_type",
            &params.search_events_params.event_type,
        );

        apply_string_query(&mut query, "status", &params.search_events_params.status);

        apply_datetime_query(
            &mut query,
            "updated_at",
            &params.search_events_params.updated_at,
        );

        if let Some(metadata) = params.search_events_params.metadata {
            query.push(" AND e.metadata @> ");
            query.push_bind(Json(metadata.clone()));
        }

        let rows = query.build().fetch_all(&self.pool).await.inspect_err(|e| {
            error!("Search events failed. DB returned error: {:?}", e);
        })?;

        let events_raw: Vec<EventRaw> = rows
            .into_iter()
            .map(|row| EventRaw::from_row(&row))
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        events_raw
            .into_iter()
            .map(CalendarEvent::try_from)
            .collect()
    }

    #[instrument]
    async fn find_most_recently_created_service_events(
        &self,
        service_id: &ID,
        user_ids: &[ID],
    ) -> anyhow::Result<Vec<MostRecentCreatedServiceEvents>> {
        let user_ids = user_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();
        // https://github.com/launchbadge/sqlx/issues/367
        let most_recent_created_service_events = sqlx::query_as!(
            MostRecentCreatedServiceEventsRaw,
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
            service_id.as_ref(),
            &user_ids
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
                error!(
                    "Find most recently created service events for service id: {} failed. DB returned error: {:?}",
                    service_id, e
                );
            })?;

        Ok(most_recent_created_service_events
            .into_iter()
            .map(|e| e.into())
            .collect())
    }

    #[instrument]
    async fn find_by_service(
        &self,
        service_id: &ID,
        user_ids: &[ID],
        min_time: DateTime<Utc>,
        max_time: DateTime<Utc>,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let user_ids = user_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();
        sqlx::query_as!(
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
        .inspect_err(|e| {
                error!(
                    "Find calendar events for service id: {}, user_ids: {:?}, min_time: {}, max_time: {} failed. DB returned error: {:?}",
                    service_id,
                    user_ids,
                    min_time,
                    max_time,
                     e
                )})?
        .into_iter().map(|e| e.try_into()).collect()
    }

    #[instrument]
    async fn find_user_service_events(
        &self,
        user_id: &ID,
        busy: bool,
        min_time: DateTime<Utc>,
        max_time: DateTime<Utc>,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        sqlx::query_as!(
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
        .inspect_err(|e| {
                error!(
                    "Find service calendar events for user_id: {}, busy: {}, min_time: {}, max_time: {} failed. DB returned error: {:?}",
                    user_id,
                    busy,
                    min_time,
                    max_time,
                     e
                );
            })?.into_iter().map(|e| e.try_into()).collect()
    }

    #[instrument]
    async fn delete(&self, event_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM calendar_events AS c
            WHERE c.event_uid = $1
            RETURNING *
            "#,
            event_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Delete calendar event with id: {:?} failed. DB returned error: {:?}",
                event_id, e
            );
        })?
        .ok_or_else(|| anyhow::Error::msg("Unable to delete calendar event"))
        .map(|_| ())
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
        .inspect_err(|e| {
            error!(
                "Delete calendar event by service id: {:?} failed. DB returned error: {:?}",
                service_id, e
            );
        })?;
        Ok(())
    }

    #[instrument]
    async fn find_by_metadata(
        &self,
        query: MetadataFindQuery,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        sqlx::query_as!(
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
        .inspect_err(|e| {
            error!(
                "Find calendar events by metadata: {:?} failed. DB returned error: {:?}",
                query, e
            );
        })?
        .into_iter()
        .map(|e| e.try_into())
        .collect()
    }
}
