mod helpers;

use chrono::DateTime;
use helpers::setup::spawn_app;
use nittei_domain::Weekday;
use nittei_sdk::{
    CreateCalendarInput,
    CreateEventInput,
    CreateUserInput,
    GetEventsInstancesInput,
    NitteiSDK,
    RRuleOptions,
    // WeekDayRecurrence,
    ID,
};

#[actix_web::main]
#[test]
async fn test_create_event_validation() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    let admin_client = NitteiSDK::new(address, res.secret_api_key);
    let user = admin_client
        .user
        .create(CreateUserInput {
            metadata: None,
            external_id: None,
            user_id: None,
        })
        .await
        .unwrap()
        .user;

    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::UTC,
            name: None,
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    let event = admin_client
        .event
        .create(CreateEventInput {
            parent_id: None,
            external_id: None,
            group_id: None,
            title: None,
            description: None,
            event_type: None,
            location: None,
            status: nittei_domain::CalendarEventStatus::Tentative,
            all_day: None,
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60,
            reminders: Vec::new(),
            busy: None,
            recurrence: None,
            exdates: None,
            recurring_event_id: Some(ID::default()),
            original_start_time: None,
            service_id: None,
            start_time: DateTime::from_timestamp_millis(0).unwrap(),
            metadata: None,
        })
        .await;
    // We expect an error because recurring_event_id is set, but original_start_time is not
    assert!(event.is_err());
}

#[actix_web::main]
#[test]
async fn test_expand_daily_recurring_event() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    let admin_client = NitteiSDK::new(address, res.secret_api_key);
    let user = admin_client
        .user
        .create(CreateUserInput {
            metadata: None,
            external_id: None,
            user_id: None,
        })
        .await
        .unwrap()
        .user;

    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::UTC,
            name: None,
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    let event = admin_client
        .event
        .create(CreateEventInput {
            parent_id: None,
            external_id: None,
            group_id: None,
            title: None,
            description: None,
            event_type: None,
            location: None,
            status: nittei_domain::CalendarEventStatus::Tentative,
            all_day: None,
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            start_time: DateTime::from_timestamp_millis(0).unwrap(),
            duration: 1000 * 60 * 60,
            reminders: Vec::new(),
            busy: None,
            recurrence: Some(RRuleOptions {
                freq: nittei_sdk::RRuleFrequency::Daily,
                interval: 1,
                ..Default::default()
            }),
            exdates: None,
            recurring_event_id: None,
            original_start_time: None,
            service_id: None,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    // Expect event to be created
    assert!(event.recurrence.is_some());

    // Get the expanded events
    let start_time = DateTime::from_timestamp_millis(0).unwrap();
    // End time of the time frame is 6 days later
    // This means we have 7 days in total
    let end_time = DateTime::from_timestamp_millis(1000 * 60 * 60 * 24 * 6).unwrap();
    let expanded_events = admin_client
        .event
        .get_instances(GetEventsInstancesInput {
            event_id: event.id.clone(),
            start_time,
            end_time,
        })
        .await
        .unwrap();

    // Expect 7 instances
    assert_eq!(expanded_events.instances.len(), 7);

    // Expect one event per day
    let mut current_time = start_time;
    for instance in expanded_events.instances {
        assert_eq!(instance.start_time, current_time);
        current_time += chrono::Duration::days(1);
    }
}

#[actix_web::main]
#[test]
async fn test_expand_weekly_recurring_event() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    let admin_client = NitteiSDK::new(address, res.secret_api_key);
    let user = admin_client
        .user
        .create(CreateUserInput {
            metadata: None,
            external_id: None,
            user_id: None,
        })
        .await
        .unwrap()
        .user;

    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::UTC,
            name: None,
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    let event = admin_client
        .event
        .create(CreateEventInput {
            parent_id: None,
            external_id: None,
            group_id: None,
            title: None,
            description: None,
            event_type: None,
            location: None,
            status: nittei_domain::CalendarEventStatus::Tentative,
            all_day: None,
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            start_time: DateTime::from_timestamp_millis(0).unwrap(),
            duration: 1000 * 60 * 60,
            reminders: Vec::new(),
            busy: None,
            recurrence: Some(RRuleOptions {
                freq: nittei_sdk::RRuleFrequency::Weekly,
                interval: 1,
                ..Default::default()
            }),
            exdates: None,
            recurring_event_id: None,
            original_start_time: None,
            service_id: None,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    // Expect event to be created
    assert!(event.recurrence.is_some());

    // Get the expanded events
    let start_time = DateTime::from_timestamp_millis(0).unwrap();
    // End time of the time frame is 7 days
    let end_time = DateTime::from_timestamp_millis(1000 * 60 * 60 * 24 * 7).unwrap();
    // println!("{:?}", start_time);
    // println!("{:?}", end_time);
    let expanded_events = admin_client
        .event
        .get_instances(GetEventsInstancesInput {
            event_id: event.id.clone(),
            start_time,
            end_time,
        })
        .await
        .unwrap();

    // println!("{:?}", expanded_events.instances);

    // Expect 2 instances
    assert_eq!(expanded_events.instances.len(), 2);
}

#[actix_web::main]
#[test]
async fn test_expand_monthly_recurring_event() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    let admin_client = NitteiSDK::new(address, res.secret_api_key);
    let user = admin_client
        .user
        .create(CreateUserInput {
            metadata: None,
            external_id: None,
            user_id: None,
        })
        .await
        .unwrap()
        .user;

    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::UTC,
            name: None,
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    let event = admin_client
        .event
        .create(CreateEventInput {
            parent_id: None,
            external_id: None,
            group_id: None,
            title: None,
            description: None,
            event_type: None,
            location: None,
            status: nittei_domain::CalendarEventStatus::Tentative,
            all_day: None,
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            start_time: DateTime::from_timestamp_millis(0).unwrap(),
            duration: 1000 * 60 * 60,
            reminders: Vec::new(),
            busy: None,
            recurrence: Some(RRuleOptions {
                freq: nittei_sdk::RRuleFrequency::Monthly,
                interval: 1,
                ..Default::default()
            }),
            exdates: None,
            recurring_event_id: None,
            original_start_time: None,
            service_id: None,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    // Expect event to be created
    assert!(event.recurrence.is_some());

    // Get the expanded events
    let start_time = DateTime::from_timestamp_millis(0).unwrap();
    // End time of the time frame is 31 days
    let end_time = DateTime::from_timestamp_millis(1000 * 60 * 60 * 24 * 31).unwrap();
    let expanded_events = admin_client
        .event
        .get_instances(GetEventsInstancesInput {
            event_id: event.id.clone(),
            start_time,
            end_time,
        })
        .await
        .unwrap();
    // Expect 2 instances
    assert_eq!(expanded_events.instances.len(), 2);
}

#[actix_web::main]
#[test]
async fn test_expand_recurring_event_and_remove_exceptions() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");
    let admin_client = NitteiSDK::new(address, res.secret_api_key);
    let user = admin_client
        .user
        .create(CreateUserInput {
            metadata: None,
            external_id: None,
            user_id: None,
        })
        .await
        .unwrap()
        .user;

    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::UTC,
            name: None,
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    let event = admin_client
        .event
        .create(CreateEventInput {
            parent_id: None,
            external_id: None,
            group_id: None,
            title: None,
            description: None,
            event_type: None,
            location: None,
            status: nittei_domain::CalendarEventStatus::Tentative,
            all_day: None,
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            start_time: DateTime::from_timestamp_millis(0).unwrap(),
            duration: 1000 * 60 * 60,
            reminders: Vec::new(),
            busy: None,
            recurrence: Some(RRuleOptions {
                freq: nittei_sdk::RRuleFrequency::Daily,
                interval: 1,
                ..Default::default()
            }),
            exdates: None,
            recurring_event_id: None,
            original_start_time: None,
            service_id: None,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    // Expect event to be created
    assert!(event.recurrence.is_some());

    // Create an exception (replace an instance with another event)
    let exception_changed = admin_client
        .event
        .create(CreateEventInput {
            parent_id: None,
            external_id: None,
            group_id: None,
            title: None,
            description: None,
            event_type: None,
            location: None,
            status: nittei_domain::CalendarEventStatus::Tentative,
            all_day: None,
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            start_time: DateTime::from_timestamp_millis(0).unwrap(),
            duration: 1000 * 60 * 60,
            reminders: Vec::new(),
            busy: None,
            recurrence: None,
            exdates: None,
            recurring_event_id: Some(event.id.clone()),
            original_start_time: Some(DateTime::from_timestamp_millis(0).unwrap()),
            service_id: None,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    // Create a 2nd exception (remove an instance)
    let exception_removed = admin_client
        .event
        .create(CreateEventInput {
            parent_id: None,
            external_id: None,
            group_id: None,
            title: None,
            description: None,
            event_type: None,
            location: None,
            status: nittei_domain::CalendarEventStatus::Cancelled, // Cancelled status
            all_day: None,
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            start_time: DateTime::from_timestamp_millis(0).unwrap(),
            duration: 1000 * 60 * 60,
            reminders: Vec::new(),
            busy: None,
            recurrence: None,
            exdates: None,
            recurring_event_id: Some(event.id.clone()),
            original_start_time: Some(
                DateTime::from_timestamp_millis(
                    1000 * 60 * 60 * 24, // 1 day later
                )
                .unwrap(),
            ),
            service_id: None,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    // Get the expanded events
    let start_time = DateTime::from_timestamp_millis(0).unwrap();
    // End time of the time frame is 6 days
    let end_time = DateTime::from_timestamp_millis(1000 * 60 * 60 * 24 * 6).unwrap();
    let expanded_events = admin_client
        .event
        .get_instances(GetEventsInstancesInput {
            event_id: event.id.clone(),
            start_time,
            end_time,
        })
        .await
        .unwrap();

    // Expect 5 instances - 2 instances have been removed => as 1 instance removed, 1 instance changed
    assert_eq!(expanded_events.instances.len(), 5);

    // Expect the instances to not contain the changed instance
    let changed_instance = expanded_events
        .instances
        .iter()
        .find(|instance| instance.start_time == exception_changed.original_start_time.unwrap());
    assert!(changed_instance.is_none());

    // Expect the instances to not contain the removed instance
    let removed_instance = expanded_events
        .instances
        .iter()
        .find(|instance| instance.start_time == exception_removed.original_start_time.unwrap());
    assert!(removed_instance.is_none());
}
