mod helpers;

use chrono::DateTime;
use helpers::setup::spawn_app;
use nittei_domain::Weekday;
use nittei_sdk::{
    CalendarDTO,
    CreateCalendarInput,
    CreateEventInput,
    CreateUserInput,
    GetCalendarEventsInput,
    GetUserFreeBusyInput,
    MultipleFreeBusyRequestBody,
    NitteiSDK,
    ID,
};
use tokio::time::Instant;

const TIMESTAMP_FIRST_JANUARY_2024: i64 = 1704067200000; // 2024-01-01 00:00:00 UTC
const MILLISECONDS_IN_ONE_DAY: i64 = 1000 * 60 * 60 * 24;

#[cfg(test)]
async fn create_account_and_admin_client() -> (NitteiSDK, ID) {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NitteiSDK::new(address, res.secret_api_key);
    (admin_client, res.account.id)
}

#[cfg(test)]
async fn create_user(admin_client: &NitteiSDK) -> ID {
    let input = CreateUserInput {
        metadata: None,
        external_id: None,
        user_id: None,
    };
    admin_client
        .user
        .create(input)
        .await
        .expect("To create user")
        .user
        .id
}

#[cfg(test)]
async fn create_calendar(admin_client: &NitteiSDK, user_id: ID) -> CalendarDTO {
    admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id,
            timezone: chrono_tz::UTC,
            name: None,
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar
}

#[cfg(test)]
async fn create_300_events_1_month(
    admin_client: &NitteiSDK,
    user_id: ID,
    calendar_id: ID,
    month_offset: i64,
) -> Vec<ID> {
    let mut event_ids = Vec::new();
    let mut day_count = 0;
    let month_offset_millis = month_offset * 30 * MILLISECONDS_IN_ONE_DAY;
    for i in 0..300 {
        // 10 events per day, 30 days
        let day_offset_millis = day_count * MILLISECONDS_IN_ONE_DAY;

        // Event index for 10 events per day, each spaced within the range of 9 AM to 6 PM
        let events_per_day = 10;
        let event_hour = 9 + (i % events_per_day); // Ensure hour stays between 9 AM and 6 PM

        // Calculate the hour offset in milliseconds
        let hour_offset_millis = event_hour * 1000 * 60 * 60;

        let start_time_millis = TIMESTAMP_FIRST_JANUARY_2024
            + month_offset_millis
            + day_offset_millis
            + hour_offset_millis;
        let event = admin_client
            .event
            .create(CreateEventInput {
                user_id: user_id.clone(),
                calendar_id: calendar_id.clone(),
                parent_id: None,
                external_id: None,
                group_id: None,
                title: None,
                description: None,
                event_type: None,
                location: None,
                status: nittei_domain::CalendarEventStatus::Tentative,
                all_day: None,
                duration: 1000 * 60 * 60, // 1 hour
                reminders: Vec::new(),
                busy: Some(true),
                recurrence: None,
                exdates: None,
                recurring_event_id: None,
                original_start_time: None,
                service_id: None,
                start_time: DateTime::from_timestamp_millis(start_time_millis).unwrap(),
                metadata: None,
            })
            .await
            .unwrap()
            .event;
        event_ids.push(event.id);

        // Increment day count every 10 events (1 day), but not on the first iteration
        if i != 0 && i % 10 == 0 {
            day_count += 1;
        }
    }
    event_ids
}

#[actix_web::main]
#[test]
async fn loadtests_single_user() {
    if std::env::var("INCLUDE_LOAD_TESTS").is_err() {
        #[allow(clippy::print_stdout)]
        {
            // Scope is needed to avoid clippy warning
            println!("[single_user] Skipped");
        }
        return;
    }
    // Setup
    let (admin_client, _) = create_account_and_admin_client().await;

    let user_id = create_user(&admin_client).await;

    let calendar = create_calendar(&admin_client, user_id.clone()).await;

    let start = Instant::now();
    let user_events =
        create_300_events_1_month(&admin_client, user_id.clone(), calendar.id.clone(), 0).await;
    let duration = start.elapsed();

    assert_eq!(user_events.len(), 300);
    // Expect the time for creating 300 events to be less than 2 seconds
    #[allow(clippy::print_stdout)]
    {
        // Scope is needed to avoid clippy warning
        println!("[single] Time to create 300 events: {:?}", duration);
    }
    assert!(duration.as_millis() < 2000);

    // Get one event
    let event = admin_client
        .event
        .get(user_events.first().unwrap().clone())
        .await
        .unwrap()
        .event;

    assert_eq!(event.calendar_id, calendar.id);

    // Measure the time it takes to fetch the events
    let start = Instant::now();

    // Get all events
    let events = admin_client
        .calendar
        .get_events(GetCalendarEventsInput {
            calendar_id: calendar.id.clone(),
            start_time: DateTime::from_timestamp_millis(TIMESTAMP_FIRST_JANUARY_2024).unwrap(),
            end_time: DateTime::from_timestamp_millis(
                TIMESTAMP_FIRST_JANUARY_2024 + 30 * 1000 * 60 * 60 * 24,
            )
            .unwrap(),
        })
        .await
        .unwrap()
        .events;

    let duration = start.elapsed();

    assert_eq!(events.len(), 300);
    // Expect the time for fetching 300 events to be less than 1 second
    #[allow(clippy::print_stdout)]
    {
        // Scope is needed to avoid clippy warning
        println!("[single] Time to fetch 300 events: {:?}", duration);
    }
    assert!(duration.as_millis() < 1000);

    // Delete the user
    admin_client.user.delete(user_id).await.unwrap();
}

#[actix_web::main]
#[test]
async fn loadtests_multi_users() {
    if std::env::var("INCLUDE_LOAD_TESTS").is_err() {
        #[allow(clippy::print_stdout)]
        {
            // Scope is needed to avoid clippy warning
            println!("[multi_users] Skipped");
        }
        return;
    }

    // Setup
    let (admin_client, _) = create_account_and_admin_client().await;

    let mut user_ids = vec![];
    for _ in 0..10 {
        user_ids.push(create_user(&admin_client).await);
    }

    let mut calendars = vec![];
    for user_id in user_ids.iter() {
        calendars.push(create_calendar(&admin_client, user_id.clone()).await);
    }

    let start = Instant::now();
    let mut event_ids = Vec::new();
    for (user_id, calendar) in user_ids.iter().zip(calendars.iter()) {
        let user_events =
            create_300_events_1_month(&admin_client, user_id.clone(), calendar.id.clone(), 0).await;
        event_ids.extend(user_events);
    }
    let duration = start.elapsed();

    assert_eq!(event_ids.len(), 3000);
    // Expect the time for creating 3000 events to be less than 20 seconds
    #[allow(clippy::print_stdout)]
    {
        // Scope is needed to avoid clippy warning
        println!("[multiple] Time to create 3000 events: {:?}", duration);
    }
    assert!(duration.as_millis() < 20000);

    // Get one event
    let start = Instant::now();
    let event = admin_client
        .event
        .get(event_ids.first().unwrap().clone())
        .await
        .unwrap()
        .event;
    let duration = start.elapsed();
    assert!(event_ids.contains(&event.id));
    assert!(duration.as_millis() < 100);

    // Measure the time it takes to fetch the events
    let start = Instant::now();

    // Get all events
    let events = admin_client
        .calendar
        .get_events(GetCalendarEventsInput {
            calendar_id: calendars.first().unwrap().id.clone(),
            start_time: DateTime::from_timestamp_millis(TIMESTAMP_FIRST_JANUARY_2024).unwrap(),
            end_time: DateTime::from_timestamp_millis(
                TIMESTAMP_FIRST_JANUARY_2024 + 30 * 1000 * 60 * 60 * 24,
            )
            .unwrap(),
        })
        .await
        .unwrap()
        .events;

    let duration = start.elapsed();

    assert_eq!(events.len(), 300);
    // Expect the time for fetching 300 events to be less than 1 second
    #[allow(clippy::print_stdout)]
    {
        // Scope is needed to avoid clippy warning
        println!("[multiple] Time to fetch 300 events: {:?}", duration);
    }
    assert!(duration.as_millis() < 1000);

    // Get free busy of one user
    let start = Instant::now();
    let free_busy = admin_client
        .user
        .free_busy(GetUserFreeBusyInput {
            user_id: user_ids.first().unwrap().clone(),
            start_time: DateTime::from_timestamp_millis(TIMESTAMP_FIRST_JANUARY_2024).unwrap(),
            end_time: DateTime::from_timestamp_millis(
                TIMESTAMP_FIRST_JANUARY_2024 + 30 * 1000 * 60 * 60 * 24,
            )
            .unwrap(),
            calendar_ids: Some(vec![calendars.first().unwrap().id.clone()]),
        })
        .await
        .unwrap();
    let duration = start.elapsed();
    // Expect the time to freebusy 300 events to be less than 500 milliseconds
    #[allow(clippy::print_stdout)]
    {
        // Scope is needed to avoid clippy warning
        println!("[multiple] Time to freebusy 300 events: {:?}", duration);
    }
    assert!(duration.as_millis() < 500);
    // Expect the freebusy to have 30 busy periods (1 per day, from 9 AM to 6 PM)
    assert_eq!(free_busy.busy.len(), 30);

    // Get free busy of all the users
    let start = Instant::now();
    let free_busy = admin_client
        .user
        .multiple_users_free_busy(MultipleFreeBusyRequestBody {
            user_ids: user_ids.clone(),
            start_time: DateTime::from_timestamp_millis(TIMESTAMP_FIRST_JANUARY_2024).unwrap(),
            end_time: DateTime::from_timestamp_millis(
                TIMESTAMP_FIRST_JANUARY_2024 + 30 * 1000 * 60 * 60 * 24,
            )
            .unwrap(),
        })
        .await
        .unwrap();
    let duration = start.elapsed();
    // Expect the time to freebusy 3000 events to be less than 1 second
    #[allow(clippy::print_stdout)]
    {
        // Scope is needed to avoid clippy warning
        println!("[multiple] Time to freebusy 3000 events: {:?}", duration);
    }
    assert!(duration.as_millis() < 1000);
    assert_eq!(free_busy.0.keys().len(), user_ids.len());

    // Delete the users
    for user_id in user_ids.iter() {
        admin_client.user.delete(user_id.clone()).await.unwrap();
    }
}

#[actix_web::main]
#[test_log::test]
async fn loadtests_single_user_lots_of_data() {
    if std::env::var("INCLUDE_LOAD_TESTS").is_err() {
        #[allow(clippy::print_stdout)]
        {
            // Scope is needed to avoid clippy warning
            println!("[single_big] Skipped");
        }
        return;
    }

    // Setup
    let (admin_client, _) = create_account_and_admin_client().await;

    let user_id = create_user(&admin_client).await;

    let calendar = create_calendar(&admin_client, user_id.clone()).await;

    let start = Instant::now();
    // Create 6000 events each month for 12 months
    let mut events = vec![];
    for i in 0..12 {
        for _ in 0..3 {
            let events_per_month =
                create_300_events_1_month(&admin_client, user_id.clone(), calendar.id.clone(), i)
                    .await;
            events.extend(events_per_month);
        }
    }
    let duration = start.elapsed();

    #[allow(clippy::print_stdout)]
    {
        // Scope is needed to avoid clippy warning
        println!("[single_big] Time to create 10800 events: {:?}", duration);
    }

    // Measure the time it takes to fetch one event
    let start = Instant::now();
    // Get one event
    let event = admin_client
        .event
        .get(events.first().unwrap().clone())
        .await
        .unwrap()
        .event;

    assert_eq!(event.calendar_id, calendar.id);
    let duration = start.elapsed();
    assert!(events.contains(&event.id));
    assert!(duration.as_millis() < 100);

    // Measure the time it takes to fetch the events of January
    let start = Instant::now();
    // Get all events
    let events = admin_client
        .calendar
        .get_events(GetCalendarEventsInput {
            calendar_id: calendar.id.clone(),
            start_time: DateTime::from_timestamp_millis(TIMESTAMP_FIRST_JANUARY_2024).unwrap(),
            end_time: DateTime::from_timestamp_millis(
                TIMESTAMP_FIRST_JANUARY_2024 + 30 * 1000 * 60 * 60 * 24,
            )
            .unwrap(),
        })
        .await
        .unwrap()
        .events;

    let duration = start.elapsed();

    assert_eq!(events.len(), 900);
    // Expect the time for fetching 900 events to be less than 2 seconds
    #[allow(clippy::print_stdout)]
    {
        // Scope is needed to avoid clippy warning
        println!("[single_big] Time to fetch 900 events: {:?}", duration);
    }
    assert!(duration.as_millis() < 2000);

    // Delete the user
    admin_client.user.delete(user_id).await.unwrap();
}
