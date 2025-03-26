use std::convert::{TryFrom, TryInto};

use chrono::{DateTime, Utc};
use nittei_domain::{
    CalendarEvent,
    CalendarEventReminder,
    CalendarEventStatus,
    ID,
    RRuleOptions,
    TimeSpan,
};
use serde_json::Value;
use sqlx::{
    FromRow,
    PgPool,
    QueryBuilder,
    types::{Json, Uuid},
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

impl TryFrom<MostRecentCreatedServiceEventsRaw> for MostRecentCreatedServiceEvents {
    type Error = anyhow::Error;

    fn try_from(e: MostRecentCreatedServiceEventsRaw) -> anyhow::Result<Self> {
        Ok(Self {
            user_id: e.user_uid.into(),
            created: e
                .created
                .map(|c| {
                    DateTime::from_timestamp_millis(c).ok_or(anyhow::anyhow!(
                        "Unable to convert created timestamp to DateTime"
                    ))
                })
                // If the created timestamp is None, return None
                // If we got an error in the internal Result, return it
                .transpose()?,
        })
    }
}

#[derive(Debug, FromRow, Clone)]
struct EventRaw {
    event_uid: Uuid,
    calendar_uid: Uuid,
    user_uid: Option<Uuid>,
    user_uid_from_user: Uuid,
    account_uid: Option<Uuid>,
    account_uid_from_user: Uuid,
    external_parent_id: Option<String>,
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
    recurrence_jsonb: Option<Value>,
    recurring_until: Option<DateTime<Utc>>,
    exdates: Vec<DateTime<Utc>>,
    recurring_event_uid: Option<Uuid>,
    original_start_time: Option<DateTime<Utc>>,
    reminders: Option<Value>,
    reminders_jsonb: Option<Value>,
    service_uid: Option<Uuid>,
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
            user_id: e.user_uid_from_user.into(),
            account_id: e.account_uid_from_user.into(),
            calendar_id: e.calendar_uid.into(),
            external_parent_id: e.external_parent_id,
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
            created: DateTime::from_timestamp_millis(e.created).ok_or(anyhow::anyhow!(
                "Unable to convert created timestamp to DateTime"
            ))?,
            updated: DateTime::from_timestamp_millis(e.updated).ok_or(anyhow::anyhow!(
                "Unable to convert updated timestamp to DateTime"
            ))?,
            recurrence,
            recurring_until: e.recurring_until,
            exdates: e.exdates,
            recurring_event_id: e.recurring_event_uid.map(|id| id.into()),
            original_start_time: e.original_start_time,
            reminders,
            service_id: e.service_uid.map(|id| id.into()),
            metadata: serde_json::from_value(e.metadata)?,
        })
    }
}

#[async_trait::async_trait]
impl IEventRepo for PostgresEventRepo {
    #[instrument(name = "calendar_event::insert")]
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        let status: String = e.status.clone().into();
        let recurrence = if e.recurrence.is_some() {
            Some(serde_json::to_value(&e.recurrence)?)
        } else {
            None
        };
        sqlx::query!(
            r#"
            INSERT INTO calendar_events(
                event_uid,
                account_uid,
                user_uid,
                calendar_uid,
                external_parent_id,
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
                recurrence_jsonb,
                recurring_until,
                exdates,
                recurring_event_uid,
                original_start_time,
                reminders,
                reminders_jsonb,
                service_uid,
                metadata
            )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
            "#,
            e.id.as_ref(),
            e.account_id.as_ref(),
            e.user_id.as_ref(),
            e.calendar_id.as_ref(),
            e.external_parent_id,
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
            e.created.timestamp_millis(),
            e.updated.timestamp_millis(),
            Json(&e.recurrence) as _,
            &recurrence as _,
            e.recurring_until,
            &e.exdates,
            e.recurring_event_id.as_ref().map(|id| id.as_ref()),
            e.original_start_time,
            Json(&e.reminders) as _,
            Json(&e.reminders) as _,
            e.service_id.as_ref().map(|id| id.as_ref()),
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

    #[instrument(name = "calendar_event::save", fields(event_uid = %e.id))]
    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()> {
        let status: String = e.status.clone().into();
        let recurrence = if e.recurrence.is_some() {
            Some(serde_json::to_value(&e.recurrence)?)
        } else {
            None
        };
        sqlx::query!(
            r#"
            UPDATE calendar_events SET
                external_parent_id = $2,
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
                recurrence_jsonb = $17,
                recurring_until = $18,
                exdates = $19,
                recurring_event_uid = $20,
                original_start_time = $21,
                reminders = $22,
                reminders_jsonb = $23,
                service_uid = $24,
                metadata = $25
            WHERE event_uid = $1
            "#,
            e.id.as_ref(),
            e.external_parent_id,
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
            e.created.timestamp_millis(),
            e.updated.timestamp_millis(),
            Json(&e.recurrence) as _,
            &recurrence as _,
            e.recurring_until,
            &e.exdates,
            e.recurring_event_id.as_ref().map(|id| id.as_ref()),
            e.original_start_time,
            Json(&e.reminders) as _,
            Json(&e.reminders) as _,
            e.service_id.as_ref().map(|id| id.as_ref()),
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

    #[instrument(name = "calendar_event::find", fields(event_uid = %event_uid))]
    async fn find(&self, event_uid: &ID) -> anyhow::Result<Option<CalendarEvent>> {
        sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.event_uid = $1
            "#,
            event_uid.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Find calendar event with id: {:?} failed. DB returned error: {:?}",
                event_uid, err
            );
        })?
        .map(|e| e.try_into())
        .transpose()
    }

    /// Find events by their recurring_event_ids during a timespan
    /// This is used to find the exceptions for recurring events
    #[instrument(name = "calendar_event::find_by_recurring_event_ids_for_timespan", fields(recurring_event_ids = ?recurring_event_ids, timespan = ?timespan))]
    async fn find_by_recurring_event_ids_for_timespan(
        &self,
        recurring_event_ids: &[ID],
        timespan: TimeSpan,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let recurring_event_ids = recurring_event_ids
            .iter()
            .map(|id| *id.as_ref())
            .collect::<Vec<_>>();
        sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.recurring_event_uid = ANY($1) AND e.original_start_time >= $2 AND e.original_start_time <= $3
            "#,
            &recurring_event_ids,
            timespan.start(),
            timespan.end(),
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Find calendar events with recurring_event_ids: {:?} failed. DB returned error: {:?}",
                recurring_event_ids, err
            );
        })?
        .into_iter()
        .map(|e| e.try_into())
        .collect()
    }

    /// Find a calendar event by its id or recurring_event_id
    /// For normal events, this will return a Vec with one element
    /// For recurring event, this can return the event + the exceptions
    /// If the event is an exception, it will only return a Vec with the exception
    #[instrument(name = "calendar_event::find_by_id_and_recurring_event_id", fields(event_uid = %event_id))]
    async fn find_by_id_and_recurring_event_id(
        &self,
        event_id: &ID,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE e.event_uid = $1 OR e.recurring_event_uid = $1
            "#,
            event_id.as_ref(),
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Find calendar event with id and recurring_event_id: {:?} failed. DB returned error: {:?}",
                event_id, err
            );
        })?
        .into_iter()
        .map(|e| e.try_into())
        .collect()
    }

    /// Find events by their external_id
    #[instrument(name = "calendar_event::get_by_external_id", fields(account_uid = %account_uid, external_id = %external_id))]
    async fn get_by_external_id(
        &self,
        account_uid: &ID,
        external_id: &str,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.account_uid = $1 AND e.external_id = $2
            "#,
            account_uid.as_ref(),
            external_id,
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Find calendar event with external_id: {:?} failed. DB returned error: {:?}",
                external_id, err
            );
        })?
        .into_iter()
        .map(|e| e.try_into())
        .collect()
    }

    /// Find events by their external_id
    #[instrument(name = "calendar_event::find_many_by_external_ids", fields(account_uid = ?account_uid, external_ids = ?external_ids))]
    async fn find_many_by_external_ids(
        &self,
        account_uid: &ID,
        external_ids: &[String],
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.account_uid = $1 AND e.external_id = any($2)
            "#,
            account_uid.as_ref(),
            external_ids,
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|err| {
            error!(
                "Find calendar events with external_ids: {:?} failed. DB returned error: {:?}",
                external_ids, err
            );
        })?
        .into_iter()
        .map(|e| e.try_into())
        .collect()
    }

    #[instrument(name = "calendar_event::find_many", fields(event_ids = ?event_ids))]
    async fn find_many(&self, event_ids: &[ID]) -> anyhow::Result<Vec<CalendarEvent>> {
        let ids = event_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();
        sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
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

    /// Find events by their calendar_id
    #[instrument(name = "calendar_event::find_by_calendar", fields(calendar_id = %calendar_id))]
    async fn find_by_calendar(
        &self,
        calendar_id: &ID,
        timespan: Option<nittei_domain::TimeSpan>,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        if let Some(timespan) = timespan {
            sqlx::query_as!(
                EventRaw,
                r#"
                    SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
                    INNER JOIN calendars AS c
                        ON c.calendar_uid = e.calendar_uid
                    INNER JOIN users AS u
                        ON u.user_uid = c.user_uid
                    WHERE e.calendar_uid = $1
                    AND (
                        (e.start_time <= $2 AND e.end_time >= $3)
                        OR
                        (e.start_time < $2 AND e.recurrence::text <> 'null' AND (e.recurring_until IS NULL OR e.recurring_until > $3))
                    )
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
                    SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
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

    /// Find events by multiple calendar_ids
    #[instrument(name = "calendar_event::find_by_calendars", fields(calendar_ids = ?calendar_ids, timespan = ?timespan))]
    async fn find_by_calendars(
        &self,
        calendar_ids: &[ID],
        timespan: nittei_domain::TimeSpan,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let calendar_ids = calendar_ids
            .iter()
            .map(|id| *id.as_ref())
            .collect::<Vec<_>>();
        sqlx::query_as!(
            EventRaw,
            r#"
                    SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
                    INNER JOIN calendars AS c
                        ON c.calendar_uid = e.calendar_uid
                    INNER JOIN users AS u
                        ON u.user_uid = c.user_uid
                    WHERE e.calendar_uid  = any($1)
                    AND (
                        (e.start_time <= $2 AND e.end_time >= $3)
                        OR 
                        (e.start_time < $2 AND e.recurrence::text <> 'null' AND (e.recurring_until IS NULL OR e.recurring_until > $3))
                    )
                    "#,
            &calendar_ids,
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

    /// Find events and recurring events for users
    /// Events need to be "busy" and with the status "confirmed"
    ///
    /// It excludes events that have an original_start_time
    /// This is used to find the normal events and the recurring events for a user
    #[instrument(name = "calendar_event::find_events_and_recurring_events_for_users_for_timespan", fields(user_ids = ?user_ids, timespan = ?timespan, include_tentative = %include_tentative, include_non_busy = %include_non_busy))]
    async fn find_events_and_recurring_events_for_users_for_timespan(
        &self,
        user_ids: &[ID],
        timespan: TimeSpan,
        include_tentative: bool,
        include_non_busy: bool,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let user_ids = user_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();
        let expected_busy: Vec<bool> = if include_non_busy {
            vec![true, false]
        } else {
            vec![true]
        };
        let expected_status: Vec<String> = if include_tentative {
            vec![
                CalendarEventStatus::Tentative.into(),
                CalendarEventStatus::Confirmed.into(),
            ]
        } else {
            vec![CalendarEventStatus::Confirmed.into()]
        };
        sqlx::query_as!(
            EventRaw,
                r#"
                    SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
                    INNER JOIN calendars AS c
                        ON c.calendar_uid = e.calendar_uid
                    INNER JOIN users AS u
                        ON u.user_uid = c.user_uid
                    WHERE u.user_uid = any($1)
                    AND (
                        (e.start_time < $2 AND e.end_time > $3)
                        OR
                        (e.start_time < $2 AND e.recurrence::text <> 'null' AND (e.recurring_until IS NULL OR e.recurring_until > $3))
                    )
                    AND busy = any($4)
                    AND status = any($5)
                    AND e.original_start_time IS NULL
                    "#,
            &user_ids,
            timespan.end(),
            timespan.start(),
            &expected_busy,
            &expected_status,
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find calendar events for user ids: {:?} failed. DB returned error: {:?}",
                user_ids, e
            );
        })?
        .into_iter()
        .map(|e| e.try_into())
        .collect()
    }

    /// Find events and recurring events for calendars
    /// Events need to be "busy" and with the status "confirmed"
    ///
    /// The parameter `include_tentative` is used to include events with the status "tentative"
    ///
    /// This is useful for the free/busy query
    #[instrument(name = "calendar_event::find_busy_events_and_recurring_events_for_calendars", fields(calendar_ids = ?calendar_ids, timespan = ?timespan, include_tentative = %include_tentative))]
    async fn find_busy_events_and_recurring_events_for_calendars(
        &self,
        calendar_ids: &[ID],
        timespan: nittei_domain::TimeSpan,
        include_tentative: bool,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let calendar_ids = calendar_ids
            .iter()
            .map(|id| *id.as_ref())
            .collect::<Vec<_>>();
        let expected_status: Vec<String> = if include_tentative {
            vec![
                CalendarEventStatus::Tentative.into(),
                CalendarEventStatus::Confirmed.into(),
            ]
        } else {
            vec![CalendarEventStatus::Confirmed.into()]
        };
        sqlx::query_as!(
            EventRaw,
            r#"
                    SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
                    INNER JOIN calendars AS c
                        ON c.calendar_uid = e.calendar_uid
                    INNER JOIN users AS u
                        ON u.user_uid = c.user_uid
                    WHERE e.calendar_uid  = any($1)
                    AND (
                        (e.start_time < $2 AND e.end_time > $3)
                        OR
                        (e.start_time < $2 AND e.recurrence::text <> 'null' AND (e.recurring_until IS NULL OR e.recurring_until > $3))
                    )
                    AND busy = true
                    AND status = any($4)
                    "#,
            &calendar_ids,
            timespan.end(),
            timespan.start(),
            expected_status.as_slice()
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
    #[instrument(name = "calendar_event::search_events_for_user", fields(params = ?params))]
    async fn search_events_for_user(
        &self,
        params: SearchEventsForUserParams,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.user_uid = "#,
        );

        query.push_bind::<Uuid>(params.user_id.into());

        apply_id_query(
            &mut query,
            "e",
            "event_uid",
            &params.search_events_params.event_uid,
        );

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
            "external_id",
            &params.search_events_params.external_id,
        );

        apply_string_query(
            &mut query,
            "external_parent_id",
            &params.search_events_params.external_parent_id,
        );

        apply_datetime_query(
            &mut query,
            "start_time",
            &params.search_events_params.start_time,
            false,
        );

        apply_datetime_query(
            &mut query,
            "end_time",
            &params.search_events_params.end_time,
            false,
        );

        apply_string_query(
            &mut query,
            "event_type",
            &params.search_events_params.event_type,
        );

        apply_string_query(&mut query, "status", &params.search_events_params.status);

        apply_id_query(
            &mut query,
            "e",
            "recurring_event_uid",
            &params.search_events_params.recurring_event_uid,
        );

        apply_datetime_query(
            &mut query,
            "original_start_time",
            &params.search_events_params.original_start_time,
            false,
        );

        if let Some(is_recurring) = params.search_events_params.is_recurring {
            query.push(format!(
                " AND e.recurrence::text {} 'null'",
                if is_recurring { "<>" } else { "=" },
            ));
        }

        if let Some(metadata) = params.search_events_params.metadata {
            query.push(" AND e.metadata @> ");
            query.push_bind(Json(metadata.clone()));
        }

        apply_datetime_query(
            &mut query,
            "created",
            &params.search_events_params.created_at,
            true,
        );

        apply_datetime_query(
            &mut query,
            "updated",
            &params.search_events_params.updated_at,
            true,
        );

        // Sort if needed
        if let Some(sort) = params.sort {
            query.push(" ORDER BY ");
            query.push(match sort {
                nittei_domain::CalendarEventSort::StartTimeAsc => "start_time ASC",
                nittei_domain::CalendarEventSort::StartTimeDesc => "start_time DESC",
                nittei_domain::CalendarEventSort::EndTimeAsc => "end_time ASC",
                nittei_domain::CalendarEventSort::EndTimeDesc => "end_time DESC",
                nittei_domain::CalendarEventSort::CreatedAsc => "created ASC",
                nittei_domain::CalendarEventSort::CreatedDesc => "created DESC",
                nittei_domain::CalendarEventSort::UpdatedAsc => "updated ASC",
                nittei_domain::CalendarEventSort::UpdatedDesc => "updated DESC",
                nittei_domain::CalendarEventSort::EventUidAsc => "event_uid ASC",
                nittei_domain::CalendarEventSort::EventUidDesc => "event_uid DESC",
            });
        }

        // Limit if needed
        if let Some(limit) = params.limit {
            query.push(" LIMIT ");
            query.push(format!("{}", limit));
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
    #[instrument(name = "calendar_event::search_events_for_account", fields(params = ?params))]
    async fn search_events_for_account(
        &self,
        params: SearchEventsForAccountParams,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
            INNER JOIN calendars AS c
                ON c.calendar_uid = e.calendar_uid
            INNER JOIN users AS u
                ON u.user_uid = c.user_uid
            WHERE u.account_uid = "#,
        );

        query.push_bind::<Uuid>(params.account_id.into());

        apply_id_query(
            &mut query,
            "e",
            "event_uid",
            &params.search_events_params.event_uid,
        );

        apply_id_query(
            &mut query,
            "u",
            "user_uid",
            &params.search_events_params.user_uid,
        );

        apply_string_query(
            &mut query,
            "external_id",
            &params.search_events_params.external_id,
        );

        apply_string_query(
            &mut query,
            "external_parent_id",
            &params.search_events_params.external_parent_id,
        );

        apply_datetime_query(
            &mut query,
            "start_time",
            &params.search_events_params.start_time,
            false,
        );

        apply_datetime_query(
            &mut query,
            "end_time",
            &params.search_events_params.end_time,
            false,
        );

        apply_string_query(
            &mut query,
            "event_type",
            &params.search_events_params.event_type,
        );

        apply_string_query(&mut query, "status", &params.search_events_params.status);

        apply_id_query(
            &mut query,
            "e",
            "recurring_event_uid",
            &params.search_events_params.recurring_event_uid,
        );

        apply_datetime_query(
            &mut query,
            "original_start_time",
            &params.search_events_params.original_start_time,
            false,
        );

        if let Some(is_recurring) = params.search_events_params.is_recurring {
            query.push(format!(
                " AND e.recurrence::text {} 'null'",
                if is_recurring { "<>" } else { "=" },
            ));
        }

        if let Some(metadata) = params.search_events_params.metadata {
            query.push(" AND e.metadata @> ");
            query.push_bind(Json(metadata.clone()));
        }

        apply_datetime_query(
            &mut query,
            "created",
            &params.search_events_params.created_at,
            true,
        );

        apply_datetime_query(
            &mut query,
            "updated",
            &params.search_events_params.updated_at,
            true,
        );

        // Sort if needed
        if let Some(sort) = params.sort {
            query.push(" ORDER BY ");
            query.push(match sort {
                nittei_domain::CalendarEventSort::StartTimeAsc => "start_time ASC",
                nittei_domain::CalendarEventSort::StartTimeDesc => "start_time DESC",
                nittei_domain::CalendarEventSort::EndTimeAsc => "end_time ASC",
                nittei_domain::CalendarEventSort::EndTimeDesc => "end_time DESC",
                nittei_domain::CalendarEventSort::CreatedAsc => "created ASC",
                nittei_domain::CalendarEventSort::CreatedDesc => "created DESC",
                nittei_domain::CalendarEventSort::UpdatedAsc => "updated ASC",
                nittei_domain::CalendarEventSort::UpdatedDesc => "updated DESC",
                nittei_domain::CalendarEventSort::EventUidAsc => "event_uid ASC",
                nittei_domain::CalendarEventSort::EventUidDesc => "event_uid DESC",
            });
        }

        // Limit if needed
        if let Some(limit) = params.limit {
            query.push(" LIMIT ");
            query.push(format!("{}", limit));
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

    #[instrument(name = "calendar_event::find_most_recently_created_service_events", fields(service_id = %service_id, user_ids = ?user_ids))]
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
                SELECT DISTINCT ON (c.user_uid) c.user_uid, e.created
                FROM calendar_events AS e
                INNER JOIN calendars AS c
                    ON c.calendar_uid = e.calendar_uid
                WHERE service_uid = $1
                ORDER BY c.user_uid, created DESC
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

        most_recent_created_service_events
            .into_iter()
            .map(MostRecentCreatedServiceEvents::try_from)
            .collect()
    }

    #[instrument(name = "calendar_event::find_by_service", fields(service_id = %service_id, user_ids = ?user_ids, min_time = %min_time, max_time = %max_time))]
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
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
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

    #[instrument(name = "calendar_event::find_user_service_events", fields(user_id = %user_id, busy = %busy, min_time = %min_time, max_time = %max_time))]
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
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
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

    /// Delete a calendar event by its uid
    #[instrument(name = "calendar_event::delete", fields(event_uid = %event_uid))]
    async fn delete(&self, event_uid: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM calendar_events AS c
            WHERE c.event_uid = $1
            RETURNING *
            "#,
            event_uid.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Delete calendar event with id: {:?} failed. DB returned error: {:?}",
                event_uid, e
            );
        })?
        .ok_or_else(|| anyhow::Error::msg("Unable to delete calendar event"))
        .map(|_| ())
    }

    /// Delete multiple calendar events by their uids
    #[instrument(name = "calendar_event::delete_many", fields(event_ids = ?event_ids))]
    async fn delete_many(&self, event_ids: &[ID]) -> anyhow::Result<()> {
        let ids = event_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();
        sqlx::query!(
            r#"
            DELETE FROM calendar_events AS c
            WHERE c.event_uid = ANY($1)
            "#,
            &ids
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Delete calendar events with ids: {:?} failed. DB returned error: {:?}",
                event_ids, e
            );
        })?;
        Ok(())
    }

    /// Delete calendar events by their service uid
    #[instrument(name = "calendar_event::delete_by_service", fields(service_id = %service_id))]
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

    /// Find calendar events by their metadata
    #[instrument(name = "calendar_event::find_by_metadata", fields(query = ?query))]
    async fn find_by_metadata(
        &self,
        query: MetadataFindQuery,
    ) -> anyhow::Result<Vec<CalendarEvent>> {
        sqlx::query_as!(
            EventRaw,
            r#"
            SELECT e.*, u.user_uid AS user_uid_from_user, u.account_uid AS account_uid_from_user FROM calendar_events AS e
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
