use std::collections::HashMap;

use axum::{Extension, Json};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use nittei_api_structs::get_events_for_users_in_time_range::*;
use nittei_domain::{
    Account,
    EventWithInstances,
    ID,
    TimeSpan,
    expand_event_and_remove_exceptions,
    generate_map_exceptions_original_start_times,
};
use nittei_infra::NitteiContext;
use nittei_utils::config::APP_CONFIG;
use tracing::error;

use crate::{
    error::NitteiError,
    shared::usecase::{UseCase, execute},
};

#[utoipa::path(
    post,
    tag = "Event",
    path = "/api/v1/events/timespan",
    summary = "Get events for users in a time range (admin only)",
    request_body(
        content = GetEventsForUsersInTimeSpanBody,
    ),
    responses(
        (status = 200, body = GetEventsForUsersInTimeSpanAPIResponse)
    )
)]
/// Get events for users in a time range
///
/// Optionally, it can generate the instances of the recurring events
pub async fn get_events_for_users_in_time_range_controller(
    Extension(account): Extension<Account>,
    Extension(ctx): Extension<NitteiContext>,
    body: Valid<Json<GetEventsForUsersInTimeSpanBody>>,
) -> Result<Json<GetEventsForUsersInTimeSpanAPIResponse>, NitteiError> {
    let usecase = GetEventsForUsersInTimeRangeUseCase {
        account_id: account.id,
        user_ids: body.user_ids.clone(),
        start_time: body.start_time,
        end_time: body.end_time,
        generate_instances_for_recurring: body.generate_instances_for_recurring.unwrap_or(false),
        include_tentative: body.include_tentative.unwrap_or(false),
        include_non_busy: body.include_non_busy.unwrap_or(false),
    };

    execute(usecase, &ctx)
        .await
        .map(|events| Json(GetEventsForUsersInTimeSpanAPIResponse::new(events.events)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetEventsForUsersInTimeRangeUseCase {
    pub account_id: ID,
    pub user_ids: Vec<ID>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub generate_instances_for_recurring: bool,
    pub include_tentative: bool,
    pub include_non_busy: bool,
}

#[derive(Debug)]
pub struct UseCaseResponse {
    pub events: Vec<EventWithInstances>,
}

#[derive(Debug)]
pub enum UseCaseError {
    InternalError,
    NotFound(String, String),
    InvalidTimespan,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::InvalidTimespan => {
                Self::BadClientData("The provided start_ts and end_ts are invalid".into())
            }
            UseCaseError::NotFound(entity, event_id) => Self::NotFound(format!(
                "The {} with id: {}, was not found.",
                entity, event_id
            )),
        }
    }
}

#[async_trait::async_trait]
impl UseCase for GetEventsForUsersInTimeRangeUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEventsForUsersInTimeRange";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<UseCaseResponse, UseCaseError> {
        let timespan = TimeSpan::new(self.start_time, self.end_time);
        if timespan.greater_than(APP_CONFIG.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        let calendars_map = if self.generate_instances_for_recurring {
            let calendars = ctx
                .repos
                .calendars
                .find_for_users(&self.user_ids)
                .await
                .map_err(|e| {
                    error!("[get_events_for_users_in_time_range] Got an error while finding calendars {:?}", e);
                    UseCaseError::InternalError
                })?;
            calendars
                .into_iter()
                .map(|calendar| (calendar.id.clone(), calendar))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        // Execute in parallel
        // Get the normal events
        // And the recurring events that are active for the users during the timespan
        let (normal_events, recurring_events) = tokio::join!(
            ctx.repos.events.find_events_for_users_for_timespan(
                &self.user_ids,
                timespan.clone(),
                self.include_tentative,
                self.include_non_busy,
            ),
            ctx.repos
                .events
                .find_recurring_events_for_users_for_timespan(
                    &self.user_ids,
                    timespan.clone(),
                    self.include_tentative,
                    self.include_non_busy,
                ),
        );

        let normal_events = match normal_events {
            Ok(events) => events
                .into_iter()
                .filter(|event| event.account_id == self.account_id)
                .collect::<Vec<_>>(),
            Err(err) => {
                error!(
                    "[get_events_for_users_in_time_range] Got an error while finding events {:?}",
                    err
                );
                return Err(UseCaseError::InternalError);
            }
        };

        let recurring_events = match recurring_events {
            Ok(events) => events
                .into_iter()
                .filter(|event| event.account_id == self.account_id)
                .collect::<Vec<_>>(),
            Err(err) => {
                error!(
                    "[get_events_for_users_in_time_range] Got an error while finding recurring events {:?}",
                    err
                );
                return Err(UseCaseError::InternalError);
            }
        };

        // Get the recurring events uids
        let recurring_events_uids = recurring_events
            .iter()
            .map(|event| event.id.clone())
            .collect::<Vec<_>>();

        // Get the exceptions for the recurring events
        let exceptions = ctx
            .repos
            .events
            .find_by_recurring_event_ids_for_timespan(&recurring_events_uids, timespan.clone())
            .await
            .map_err(|err| {
                error!(
                    "[get_events_for_users_in_time_range] Got an error while finding events {:?}",
                    err
                );
                UseCaseError::InternalError
            })?;

        // Create a map of recurrence_id to events (exceptions)
        // This is used to remove exceptions from the expanded events
        let map_recurring_event_id_to_exceptions =
            generate_map_exceptions_original_start_times(&exceptions);

        // Concat the normal events and the recurring events with the exceptions
        let all_events = [normal_events, recurring_events, exceptions].concat();

        let events_to_return = if !self.generate_instances_for_recurring {
            // If we don't want to generate instances, we just return the events
            all_events
                .into_iter()
                .map(|event| EventWithInstances {
                    event,
                    instances: vec![],
                })
                .collect::<Vec<_>>()
        } else {
            // For each event, expand it and keep the instances next to the event
            all_events
                .into_iter()
                .map(|event| {
                    let calendar = calendars_map.get(&event.calendar_id).ok_or_else(|| {
                        UseCaseError::NotFound(
                            "Calendar".to_string(),
                            event.calendar_id.to_string(),
                        )
                    })?;
                    // Get the exceptions of the event
                    let exceptions = map_recurring_event_id_to_exceptions
                        .get(&event.id)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]);

                    // Expand the event and remove the exceptions
                    let timespan = timespan.clone();

                    let instances =
                        expand_event_and_remove_exceptions(calendar, &event, exceptions, timespan)
                            .map_err(|e| {
                                error!("[get_events_for_users_in_time_range] Got an error while expanding an event {:?}", e);
                                UseCaseError::InternalError
                            })?;

                    Ok(EventWithInstances { event, instances })
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .collect::<Vec<_>>()
        };

        Ok(UseCaseResponse {
            events: events_to_return,
        })
    }
}

#[cfg(test)]
mod test {
    use chrono::prelude::*;
    use nittei_domain::{
        Account,
        Calendar,
        CalendarEvent,
        CalendarEventStatus,
        RRuleFrequency,
        RRuleOptions,
        User,
    };
    use nittei_infra::setup_context;

    use super::*;

    struct TestContext {
        ctx: NitteiContext,
        account: Account,
        calendar: Calendar,
        user: User,
    }

    async fn setup() -> TestContext {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id, None, None);
        ctx.repos.calendars.insert(&calendar).await.unwrap();

        TestContext {
            ctx,
            account,
            calendar,
            user,
        }
    }

    async fn create_event(ctx: &NitteiContext, event: CalendarEvent) {
        ctx.repos.events.insert(&event).await.unwrap();
    }

    #[tokio::test]
    async fn fetches_events_for_users_in_time_range() {
        let TestContext {
            ctx,
            account,
            user,
            calendar,
        } = setup().await;

        // Create a normal event
        let normal_event_id = ID::new_v4();
        let normal_event = CalendarEvent {
            id: normal_event_id.clone(),
            title: Some("Test".to_string()),
            status: CalendarEventStatus::Confirmed,
            all_day: false,
            start_time: DateTime::parse_from_rfc3339("2025-02-01T11:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            duration: 3600000, // 1 hour
            end_time: DateTime::parse_from_rfc3339("2025-02-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            busy: true,
            created: DateTime::from_timestamp_millis(0).unwrap(),
            updated: DateTime::from_timestamp_millis(0).unwrap(),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: account.id.clone(),
            ..Default::default()
        };

        create_event(&ctx, normal_event).await;

        // Create a recurring event
        let recurring_event_id = ID::new_v4();
        let recurring_event = CalendarEvent {
            id: recurring_event_id.clone(),
            title: Some("Test".to_string()),
            status: CalendarEventStatus::Confirmed,
            all_day: false,
            start_time: DateTime::parse_from_rfc3339("2025-01-01T14:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            duration: 3600000, // 1 hour
            busy: true,
            end_time: DateTime::parse_from_rfc3339("2025-01-01T15:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            created: DateTime::from_timestamp_millis(0).unwrap(),
            updated: DateTime::from_timestamp_millis(0).unwrap(),
            recurrence: Some(RRuleOptions {
                freq: RRuleFrequency::Daily,
                interval: 1,
                ..Default::default()
            }),
            exdates: vec![],
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: account.id.clone(),
            ..Default::default()
        };

        create_event(&ctx, recurring_event).await;

        // Create a cancelled exception for the recurring event
        let exception_id = ID::new_v4();
        let exception_event = CalendarEvent {
            id: exception_id.clone(),
            title: Some("Test".to_string()),
            status: CalendarEventStatus::Cancelled,
            all_day: false,
            start_time: DateTime::parse_from_rfc3339("2025-02-03T14:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            duration: 3600000, // 1 hour
            end_time: DateTime::parse_from_rfc3339("2025-02-03T15:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            busy: true,
            created: DateTime::from_timestamp_millis(0).unwrap(),
            updated: DateTime::from_timestamp_millis(0).unwrap(),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: account.id.clone(),
            recurring_event_id: Some(recurring_event_id.clone()),
            original_start_time: Some(
                DateTime::parse_from_rfc3339("2025-02-01T14:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ..Default::default()
        };

        create_event(&ctx, exception_event).await;

        // Create a moved exception for the recurring event
        let moved_exception_id = ID::new_v4();
        let moved_exception_event = CalendarEvent {
            id: moved_exception_id.clone(),
            title: Some("Test".to_string()),
            status: CalendarEventStatus::Confirmed,
            all_day: false,
            start_time: DateTime::parse_from_rfc3339("2025-02-07T09:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            duration: 3600000, // 1 hour
            end_time: DateTime::parse_from_rfc3339("2025-02-07T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            busy: true,
            created: DateTime::from_timestamp_millis(0).unwrap(),
            updated: DateTime::from_timestamp_millis(0).unwrap(),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: account.id.clone(),
            recurring_event_id: Some(recurring_event_id.clone()),
            original_start_time: Some(
                DateTime::parse_from_rfc3339("2025-02-06T14:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ..Default::default()
        };

        create_event(&ctx, moved_exception_event).await;

        let mut usecase = GetEventsForUsersInTimeRangeUseCase {
            start_time: DateTime::parse_from_rfc3339("2025-02-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            end_time: DateTime::parse_from_rfc3339("2025-02-07T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            user_ids: vec![user.id.clone()],
            account_id: account.id.clone(),
            generate_instances_for_recurring: false,
            include_tentative: false,
            include_non_busy: false,
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());

        let events = res.unwrap().events;
        assert_eq!(events.len(), 4);
        assert!(events.iter().any(|e| e.event.id == normal_event_id));
        assert!(events.iter().any(|e| e.event.id == recurring_event_id));
        assert!(events.iter().any(|e| e.event.id == exception_id));
        assert!(events.iter().any(|e| e.event.id == moved_exception_id));
    }

    #[tokio::test]
    async fn fetches_events_with_instances_generated() {
        let TestContext {
            ctx,
            account,
            user,
            calendar,
        } = setup().await;

        // Create a recurring event
        let recurring_event_id = ID::new_v4();
        let recurring_event = CalendarEvent {
            id: recurring_event_id.clone(),
            title: Some("Daily Meeting".to_string()),
            status: CalendarEventStatus::Confirmed,
            all_day: false,
            start_time: DateTime::parse_from_rfc3339("2025-01-01T14:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            duration: 3600000, // 1 hour
            busy: true,
            end_time: DateTime::parse_from_rfc3339("2025-01-01T15:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            created: DateTime::from_timestamp_millis(0).unwrap(),
            updated: DateTime::from_timestamp_millis(0).unwrap(),
            recurrence: Some(RRuleOptions {
                freq: RRuleFrequency::Daily,
                interval: 1,
                ..Default::default()
            }),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: account.id.clone(),
            ..Default::default()
        };

        create_event(&ctx, recurring_event).await;

        let mut usecase = GetEventsForUsersInTimeRangeUseCase {
            start_time: DateTime::parse_from_rfc3339("2025-02-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            end_time: DateTime::parse_from_rfc3339("2025-02-03T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            user_ids: vec![user.id.clone()],
            account_id: account.id.clone(),
            generate_instances_for_recurring: true,
            include_tentative: false,
            include_non_busy: false,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());

        let events = res.unwrap().events;
        assert_eq!(events.len(), 1); // One recurring event
        let event_with_instances = &events[0];
        assert_eq!(event_with_instances.event.id, recurring_event_id);
        assert_eq!(event_with_instances.instances.len(), 2); // Should have instances for Feb 1st and 2nd
    }

    #[tokio::test]
    async fn fetches_tentative_events_when_included() {
        let TestContext {
            ctx,
            account,
            user,
            calendar,
        } = setup().await;

        // Create a tentative event
        let tentative_event_id = ID::new_v4();
        let tentative_event = CalendarEvent {
            id: tentative_event_id.clone(),
            title: Some("Maybe Meeting".to_string()),
            status: CalendarEventStatus::Tentative,
            all_day: false,
            start_time: DateTime::parse_from_rfc3339("2025-02-01T11:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            duration: 3600000,
            end_time: DateTime::parse_from_rfc3339("2025-02-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            busy: true,
            created: DateTime::from_timestamp_millis(0).unwrap(),
            updated: DateTime::from_timestamp_millis(0).unwrap(),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: account.id.clone(),
            ..Default::default()
        };

        create_event(&ctx, tentative_event).await;

        // Test with include_tentative = false
        let mut usecase = GetEventsForUsersInTimeRangeUseCase {
            start_time: DateTime::parse_from_rfc3339("2025-02-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            end_time: DateTime::parse_from_rfc3339("2025-02-02T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            user_ids: vec![user.id.clone()],
            account_id: account.id.clone(),
            generate_instances_for_recurring: false,
            include_tentative: false,
            include_non_busy: false,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().events.len(), 0); // Should not include tentative event

        // Test with include_tentative = true
        usecase.include_tentative = true;
        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let events = res.unwrap().events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.id, tentative_event_id);
    }

    #[tokio::test]
    async fn fetches_non_busy_events_when_included() {
        let TestContext {
            ctx,
            account,
            user,
            calendar,
        } = setup().await;

        // Create a non-busy event
        let non_busy_event_id = ID::new_v4();
        let non_busy_event = CalendarEvent {
            id: non_busy_event_id.clone(),
            title: Some("Out of Office".to_string()),
            status: CalendarEventStatus::Confirmed,
            all_day: false,
            start_time: DateTime::parse_from_rfc3339("2025-02-01T11:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            duration: 3600000,
            end_time: DateTime::parse_from_rfc3339("2025-02-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            busy: false,
            created: DateTime::from_timestamp_millis(0).unwrap(),
            updated: DateTime::from_timestamp_millis(0).unwrap(),
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: account.id.clone(),
            ..Default::default()
        };

        create_event(&ctx, non_busy_event).await;

        // Test with include_non_busy = false
        let mut usecase = GetEventsForUsersInTimeRangeUseCase {
            start_time: DateTime::parse_from_rfc3339("2025-02-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            end_time: DateTime::parse_from_rfc3339("2025-02-02T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            user_ids: vec![user.id.clone()],
            account_id: account.id.clone(),
            generate_instances_for_recurring: false,
            include_tentative: false,
            include_non_busy: false,
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().events.len(), 0); // Should not include non-busy event

        // Test with include_non_busy = true
        usecase.include_non_busy = true;
        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());
        let events = res.unwrap().events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.id, non_busy_event_id);
    }
}
