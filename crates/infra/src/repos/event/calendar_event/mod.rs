mod postgres;

use chrono::{DateTime, Utc};
use nittei_domain::{CalendarEvent, DateTimeQuery, IdQuery, StringQuery, TimeSpan, ID};
pub use postgres::PostgresEventRepo;

use crate::repos::shared::query_structs::MetadataFindQuery;

#[derive(Debug)]
pub struct MostRecentCreatedServiceEvents {
    pub user_id: ID,
    pub created: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SearchEventsForUserParams {
    pub user_id: ID,
    pub calendar_ids: Option<Vec<ID>>,
    pub search_events_params: SearchEventsParams,
}

#[derive(Debug, Clone)]
pub struct SearchEventsForAccountParams {
    pub account_id: ID,
    pub search_events_params: SearchEventsParams,
}

#[derive(Debug, Clone)]
pub struct SearchEventsParams {
    pub parent_id: Option<StringQuery>,
    pub group_id: Option<IdQuery>,
    pub start_time: Option<DateTimeQuery>,
    pub end_time: Option<DateTimeQuery>,
    pub status: Option<StringQuery>,
    pub event_type: Option<StringQuery>,
    pub updated_at: Option<DateTimeQuery>,
    pub metadata: Option<serde_json::Value>,
}

#[async_trait::async_trait]
pub trait IEventRepo: Send + Sync {
    async fn insert(&self, e: &CalendarEvent) -> anyhow::Result<()>;
    async fn save(&self, e: &CalendarEvent) -> anyhow::Result<()>;
    async fn find(&self, event_id: &ID) -> anyhow::Result<Option<CalendarEvent>>;
    async fn get_by_external_id(
        &self,
        account_uid: &ID,
        external_id: &str,
        include_groups: bool,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn find_many(&self, event_ids: &[ID]) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn find_by_calendar(
        &self,
        calendar_id: &ID,
        timespan: Option<&TimeSpan>,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn find_by_calendars(
        &self,
        calendar_ids: Vec<ID>,
        timespan: &TimeSpan,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn search_events_for_user(
        &self,
        params: SearchEventsForUserParams,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn search_events_for_account(
        &self,
        params: SearchEventsForAccountParams,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn find_most_recently_created_service_events(
        &self,
        service_id: &ID,
        user_ids: &[ID],
    ) -> anyhow::Result<Vec<MostRecentCreatedServiceEvents>>;
    async fn find_by_service(
        &self,
        service_id: &ID,
        user_ids: &[ID],
        min_time: DateTime<Utc>,
        max_time: DateTime<Utc>,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn find_user_service_events(
        &self,
        user_id: &ID,
        busy: bool,
        min_time: DateTime<Utc>,
        max_time: DateTime<Utc>,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
    async fn delete(&self, event_id: &ID) -> anyhow::Result<()>;
    async fn delete_by_service(&self, service_id: &ID) -> anyhow::Result<()>;
    async fn find_by_metadata(
        &self,
        query: MetadataFindQuery,
    ) -> anyhow::Result<Vec<CalendarEvent>>;
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use nittei_domain::{Account, Calendar, CalendarEvent, Entity, Service, TimeSpan, User, ID};

    use crate::{setup_context, NitteiContext};

    fn generate_default_event(account_id: &ID, calendar_id: &ID, user_id: &ID) -> CalendarEvent {
        CalendarEvent {
            account_id: account_id.clone(),
            calendar_id: calendar_id.clone(),
            user_id: user_id.clone(),
            ..Default::default()
        }
    }

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
            account,
            calendar,
            user,
            ctx,
        }
    }

    #[tokio::test]
    async fn create_and_delete() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let event = generate_default_event(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.events.insert(&event).await.is_ok());

        // Different find methods
        let get_event_res = ctx.repos.events.find(&event.id).await.unwrap().unwrap();
        assert!(get_event_res.eq(&event));
        let get_event_res = ctx
            .repos
            .events
            .find_many(&[event.id.clone()])
            .await
            .expect("To find many events");
        assert!(get_event_res[0].eq(&event));

        // Delete
        let delete_res = ctx.repos.events.delete(&event.id).await;
        assert!(delete_res.is_ok());

        // Find
        assert!(ctx.repos.events.find(&event.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn update() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let mut event = generate_default_event(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.events.insert(&event).await.is_ok());

        event.updated += 1;

        // Save
        assert!(ctx.repos.events.save(&event).await.is_ok());

        // Find
        assert!(ctx
            .repos
            .events
            .find(&event.id)
            .await
            .unwrap()
            .expect("To be event")
            .eq(&event));
    }

    #[tokio::test]
    async fn delete_by_user() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let event = generate_default_event(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.events.insert(&event).await.is_ok());

        // Delete
        assert!(ctx.repos.users.delete(&user.id).await.unwrap().is_some());

        // Find after delete
        assert!(ctx.repos.events.find(&event.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_by_calendar() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let event = generate_default_event(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.events.insert(&event).await.is_ok());

        // Delete
        assert!(ctx.repos.calendars.delete(&calendar.id).await.is_ok());

        // Find after delete
        assert!(ctx.repos.events.find(&event.id).await.unwrap().is_none());
    }

    async fn generate_event_with_time(
        account_id: &ID,
        calendar_id: &ID,
        user_id: &ID,
        service_id: Option<&ID>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        ctx: &NitteiContext,
    ) -> CalendarEvent {
        let mut event = generate_default_event(account_id, calendar_id, user_id);
        event.calendar_id = calendar_id.clone();
        event.start_time = start_time;
        event.end_time = end_time;
        event.service_id = service_id.cloned();
        ctx.repos
            .events
            .insert(&event)
            .await
            .expect("To insert event");
        event
    }

    async fn generate_event_with_time_2(
        account_id: &ID,
        calendar_id: &ID,
        user_id: &ID,
        service_id: &ID,
        created: i64,
        ctx: &NitteiContext,
    ) -> CalendarEvent {
        let mut event = generate_default_event(account_id, calendar_id, user_id);
        event.calendar_id = calendar_id.clone();
        event.service_id = Some(service_id.clone());
        event.created = created;
        ctx.repos
            .events
            .insert(&event)
            .await
            .expect("To insert event");
        event
    }

    #[tokio::test]
    async fn find_by_calendar_and_timespan() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let start_ts = 100;
        let end_ts = 200;
        // All the possible combination of intervals
        let event_1 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts - 2).unwrap(),
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            &ctx,
        )
        .await;
        let event_2 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            &ctx,
        )
        .await;
        let event_3 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            DateTime::from_timestamp_millis(start_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_4 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            &ctx,
        )
        .await;
        let event_5 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_6 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts - 1).unwrap(),
            &ctx,
        )
        .await;
        let event_7 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            &ctx,
        )
        .await;
        let event_8 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_9 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts + 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts - 1).unwrap(),
            &ctx,
        )
        .await;
        let event_10 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts + 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            &ctx,
        )
        .await;
        let event_11 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts + 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_12 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_13 = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 2).unwrap(),
            &ctx,
        )
        .await;

        let actual_events_in_timespan = vec![
            event_2.clone(),
            event_3.clone(),
            event_4.clone(),
            event_5.clone(),
            event_6.clone(),
            event_7.clone(),
            event_8.clone(),
            event_9.clone(),
            event_10.clone(),
            event_11.clone(),
            event_12.clone(),
        ];

        let mut actual_events_in_calendar = actual_events_in_timespan.clone();
        actual_events_in_calendar.push(event_1.clone());
        actual_events_in_calendar.push(event_13.clone());

        // Find
        let events_in_calendar_and_timespan = ctx
            .repos
            .events
            .find_by_calendar(
                &calendar.id,
                Some(&TimeSpan::new(
                    DateTime::from_timestamp_millis(start_ts).unwrap(),
                    DateTime::from_timestamp_millis(end_ts).unwrap(),
                )),
            )
            .await
            .expect("To get events");

        assert_eq!(
            events_in_calendar_and_timespan.len(),
            actual_events_in_timespan.len()
        );
        for actual_event in actual_events_in_timespan {
            assert!(events_in_calendar_and_timespan
                .iter()
                .any(|e| e.id() == actual_event.id()));
        }

        let events_in_calendar = ctx
            .repos
            .events
            .find_by_calendar(&calendar.id, None)
            .await
            .expect("To get events");
        assert_eq!(actual_events_in_calendar.len(), events_in_calendar.len());
        for actual_event in actual_events_in_calendar {
            assert!(events_in_calendar
                .iter()
                .any(|e| e.id() == actual_event.id()));
        }
    }

    #[tokio::test]
    async fn find_most_recently_created_service_events() {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();

        let service = Service::new(account.id.clone());
        ctx.repos.services.insert(&service).await.unwrap();
        let other_service = Service::new(account.id.clone());
        ctx.repos.services.insert(&other_service).await.unwrap();

        // User 1
        let user1 = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user1).await.unwrap();
        let calendar1 = Calendar::new(&user1.id, &account.id, None, None);
        ctx.repos.calendars.insert(&calendar1).await.unwrap();

        // User 2
        let user2 = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user2).await.unwrap();
        let calendar2 = Calendar::new(&user2.id, &account.id, None, None);
        ctx.repos.calendars.insert(&calendar2).await.unwrap();

        // User 3
        let user3 = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user3).await.unwrap();
        let calendar3 = Calendar::new(&user3.id, &account.id, None, None);
        ctx.repos.calendars.insert(&calendar3).await.unwrap();

        // User 1 has three events
        let user_1_recent_created_event = 100;
        generate_event_with_time_2(
            &account.id,
            &calendar1.id,
            &user1.id,
            &service.id,
            user_1_recent_created_event - 10,
            &ctx,
        )
        .await;
        generate_event_with_time_2(
            &account.id,
            &calendar1.id,
            &user1.id,
            &service.id,
            user_1_recent_created_event,
            &ctx,
        )
        .await;
        generate_event_with_time_2(
            &account.id,
            &calendar1.id,
            &user1.id,
            &service.id,
            user_1_recent_created_event - 5,
            &ctx,
        )
        .await;

        // User 2 has one event on this service and one in another service
        let user_2_recent_created_event = 70;
        generate_event_with_time_2(
            &account.id,
            &calendar2.id,
            &user2.id,
            &service.id,
            user_2_recent_created_event,
            &ctx,
        )
        .await;
        // Event on other service should not affect this query
        generate_event_with_time_2(
            &account.id,
            &calendar1.id,
            &user1.id,
            &other_service.id,
            user_2_recent_created_event + 5,
            &ctx,
        )
        .await;

        // User 3 has no events

        let recent_service_events = ctx
            .repos
            .events
            .find_most_recently_created_service_events(
                &service.id,
                &[user1.id.clone(), user2.id.clone(), user3.id.clone()],
            )
            .await
            .unwrap();
        assert_eq!(recent_service_events.len(), 3);
        let user1_recent_service_events = recent_service_events
            .iter()
            .find(|e| e.user_id == user1.id)
            .expect("User to be there");
        assert_eq!(
            user1_recent_service_events.created,
            Some(user_1_recent_created_event)
        );
        let user2_recent_service_events = recent_service_events
            .iter()
            .find(|e| e.user_id == user2.id)
            .expect("User to be there");
        assert_eq!(
            user2_recent_service_events.created,
            Some(user_2_recent_created_event)
        );
        let user3_recent_service_events = recent_service_events
            .iter()
            .find(|e| e.user_id == user3.id)
            .expect("User to be there");
        assert_eq!(user3_recent_service_events.created, None);
    }

    #[tokio::test]
    async fn find_by_service_and_timespan() {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();

        let service = Service::new(account.id.clone());
        ctx.repos.services.insert(&service).await.unwrap();
        let other_service = Service::new(account.id.clone());
        ctx.repos.services.insert(&other_service).await.unwrap();

        // User 1
        let user1 = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user1).await.unwrap();
        let calendar1 = Calendar::new(&user1.id, &account.id, None, None);
        ctx.repos.calendars.insert(&calendar1).await.unwrap();

        let start_ts = 100;
        let end_ts = 200;
        // All the possible combination of intervals
        let event_1 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts - 2).unwrap(),
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            &ctx,
        )
        .await;
        let event_2 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            &ctx,
        )
        .await;
        let event_3 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            DateTime::from_timestamp_millis(start_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_4 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            &ctx,
        )
        .await;
        let event_5 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts - 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_6 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts - 1).unwrap(),
            &ctx,
        )
        .await;
        let event_7 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            &ctx,
        )
        .await;
        let event_8 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_9 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts + 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts - 1).unwrap(),
            &ctx,
        )
        .await;
        let event_10 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts + 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            &ctx,
        )
        .await;
        let event_11 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts + 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_12 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            &ctx,
        )
        .await;
        let event_13 = generate_event_with_time(
            &account.id,
            &calendar1.id,
            &user1.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(end_ts + 1).unwrap(),
            DateTime::from_timestamp_millis(end_ts + 2).unwrap(),
            &ctx,
        )
        .await;

        let actual_events_in_timespan = vec![
            event_2.clone(),
            event_3.clone(),
            event_4.clone(),
            event_5.clone(),
            event_6.clone(),
            event_7.clone(),
            event_8.clone(),
            event_9.clone(),
            event_10.clone(),
            event_11.clone(),
            event_12.clone(),
        ];

        let mut actual_events_in_service = actual_events_in_timespan.clone();
        actual_events_in_service.push(event_1.clone());
        actual_events_in_service.push(event_13.clone());

        // Find
        let events_in_service_and_timespan = ctx
            .repos
            .events
            .find_by_service(
                &service.id,
                &[user1.id.clone()],
                DateTime::from_timestamp_millis(start_ts).unwrap(),
                DateTime::from_timestamp_millis(end_ts).unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            events_in_service_and_timespan.len(),
            actual_events_in_timespan.len()
        );
        for actual_event in actual_events_in_timespan {
            assert!(events_in_service_and_timespan
                .iter()
                .any(|e| e.id() == actual_event.id()));
        }

        let events_in_service_with_no_users = ctx
            .repos
            .events
            .find_by_service(
                &service.id,
                &Vec::new(),
                DateTime::from_timestamp_millis(start_ts).unwrap(),
                DateTime::from_timestamp_millis(end_ts).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(events_in_service_with_no_users.len(), 0);
    }

    #[tokio::test]
    async fn test_find_user_service_events() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let start_ts = 10;
        let end_ts = 20;

        let service = Service::new(account.id.clone());
        ctx.repos
            .services
            .insert(&service)
            .await
            .expect("To create service");

        let service_event = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            Some(&service.id),
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            &ctx,
        )
        .await;
        let _not_service_event = generate_event_with_time(
            &account.id,
            &calendar.id,
            &user.id,
            None,
            DateTime::from_timestamp_millis(start_ts).unwrap(),
            DateTime::from_timestamp_millis(end_ts).unwrap(),
            &ctx,
        )
        .await;

        let res = ctx
            .repos
            .events
            .find_user_service_events(
                &user.id,
                false,
                DateTime::from_timestamp_millis(start_ts).unwrap(),
                DateTime::from_timestamp_millis(end_ts).unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, service_event.id);
    }
}
