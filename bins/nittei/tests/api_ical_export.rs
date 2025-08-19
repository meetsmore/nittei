mod helpers;

use chrono::Utc;
use helpers::setup::spawn_app;
use nittei_domain::{CalendarEventStatus, RRuleFrequency, RRuleOptions, Weekday};
use nittei_sdk::{CreateCalendarInput, CreateEventInput, CreateUserInput, NitteiSDK};

#[tokio::test]
async fn test_export_calendar_ical_user_endpoint() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NitteiSDK::new(address.clone(), res.secret_api_key.clone());

    // Create user
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

    // Create calendar
    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::UTC,
            name: Some("Test Calendar".to_string()),
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    // Create an event within the default timespan (1 month ago)
    let event_start = Utc::now() - chrono::Duration::days(30);
    let _event = admin_client
        .event
        .create(CreateEventInput {
            external_parent_id: None,
            external_id: None,
            title: Some("Test Event".to_string()),
            description: Some("Test Description".to_string()),
            event_type: None,
            location: Some("Test Location".to_string()),
            status: CalendarEventStatus::Confirmed,
            all_day: Some(false),
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60, // 1 hour
            reminders: Vec::new(),
            busy: Some(true),
            recurrence: None,
            exdates: None,
            recurring_event_id: None,
            original_start_time: None,
            service_id: None,
            start_time: event_start,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    // Test the iCal export endpoint using the SDK client (using default timespan)
    let ical_content = admin_client
        .calendar
        .export_ical(nittei_sdk::ExportCalendarIcalInput {
            calendar_id: calendar.id.clone(),
            start_time: None, // Use default (3 months ago)
            end_time: None,   // Use default (6 months in future)
        })
        .await
        .expect("Expected to export calendar as iCal");

    // Verify iCal content structure
    assert!(ical_content.contains("BEGIN:VCALENDAR"));
    assert!(ical_content.contains("END:VCALENDAR"));
    assert!(ical_content.contains("VERSION:2.0"));
    assert!(ical_content.contains("PRODID:-//Nittei//Calendar API//EN"));

    // Verify calendar properties
    assert!(ical_content.contains("X-WR-CALNAME:Test Calendar"));
    assert!(ical_content.contains("X-WR-TIMEZONE:UTC"));

    // Verify event properties in iCal format
    assert!(ical_content.contains("SUMMARY:Test Event"));
    assert!(ical_content.contains("DESCRIPTION:Test Description"));
    assert!(ical_content.contains("LOCATION:Test Location"));
    assert!(ical_content.contains("STATUS:CONFIRMED"));
    assert!(ical_content.contains("DTSTART:"));
    assert!(ical_content.contains("DTEND:"));
}

#[tokio::test]
async fn test_export_calendar_ical_admin_endpoint() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NitteiSDK::new(address.clone(), res.secret_api_key.clone());

    // Create user
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

    // Create calendar
    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::Europe::Oslo,
            name: Some("Admin Test Calendar".to_string()),
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    // Create multiple events within the default timespan
    let event1_start = Utc::now() - chrono::Duration::days(60); // 2 months ago
    let event2_start = Utc::now() + chrono::Duration::days(30); // 1 month in future

    let _event1 = admin_client
        .event
        .create(CreateEventInput {
            external_parent_id: None,
            external_id: None,
            title: Some("First Event".to_string()),
            description: Some("First event description".to_string()),
            event_type: None,
            location: None,
            status: CalendarEventStatus::Confirmed,
            all_day: Some(false),
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 30, // 30 minutes
            reminders: Vec::new(),
            busy: Some(true),
            recurrence: None,
            exdates: None,
            recurring_event_id: None,
            original_start_time: None,
            service_id: None,
            start_time: event1_start,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    let _event2 = admin_client
        .event
        .create(CreateEventInput {
            external_parent_id: None,
            external_id: None,
            title: Some("Second Event".to_string()),
            description: None,
            event_type: None,
            location: Some("Office".to_string()),
            status: CalendarEventStatus::Tentative,
            all_day: Some(true),
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 60 * 24, // 24 hours (all day)
            reminders: Vec::new(),
            busy: Some(false),
            recurrence: None,
            exdates: None,
            recurring_event_id: None,
            original_start_time: None,
            service_id: None,
            start_time: event2_start,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    // Test the iCal export endpoint using the SDK client (using default timespan)
    let ical_content = admin_client
        .calendar
        .export_ical(nittei_sdk::ExportCalendarIcalInput {
            calendar_id: calendar.id.clone(),
            start_time: None, // Use default (3 months ago)
            end_time: None,   // Use default (6 months in future)
        })
        .await
        .expect("Expected to export calendar as iCal");

    // Verify iCal content structure
    assert!(ical_content.contains("BEGIN:VCALENDAR"));
    assert!(ical_content.contains("END:VCALENDAR"));
    assert!(ical_content.contains("VERSION:2.0"));
    assert!(ical_content.contains("PRODID:-//Nittei//Calendar API//EN"));

    // Verify calendar properties
    assert!(ical_content.contains("X-WR-CALNAME:Admin Test Calendar"));
    assert!(ical_content.contains("X-WR-TIMEZONE:Europe/Oslo"));

    // Verify both events are present in iCal
    assert!(ical_content.contains("SUMMARY:First Event"));
    assert!(ical_content.contains("DESCRIPTION:First event description"));
    assert!(ical_content.contains("STATUS:CONFIRMED"));
    assert!(ical_content.contains("DTSTART:"));
    assert!(ical_content.contains("DTEND:"));

    assert!(ical_content.contains("SUMMARY:Second Event"));
    assert!(ical_content.contains("LOCATION:Office"));
    assert!(ical_content.contains("STATUS:TENTATIVE"));
    // All-day events use DATE format
    assert!(ical_content.contains("DTSTART;VALUE=DATE:"));
    assert!(ical_content.contains("DTEND;VALUE=DATE:"));
}

#[tokio::test]
async fn test_export_calendar_ical_with_recurring_events() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NitteiSDK::new(address.clone(), res.secret_api_key.clone());

    // Create user
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

    // Create calendar
    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::UTC,
            name: Some("Recurring Events Calendar".to_string()),
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    // Create a recurring event (daily for 5 days) within the default timespan
    let recurrence = RRuleOptions {
        freq: RRuleFrequency::Daily,
        interval: 1,
        count: Some(5),
        until: None,
        bysetpos: None,
        byweekday: None,
        bymonthday: None,
        bymonth: None,
        byyearday: None,
        byweekno: None,
        weekstart: None,
    };

    let recurring_event_start = Utc::now() + chrono::Duration::days(7); // 1 week in future
    let _recurring_event = admin_client
        .event
        .create(CreateEventInput {
            external_parent_id: None,
            external_id: None,
            title: Some("Daily Meeting".to_string()),
            description: Some("Daily standup meeting".to_string()),
            event_type: None,
            location: Some("Conference Room".to_string()),
            status: CalendarEventStatus::Confirmed,
            all_day: Some(false),
            user_id: user.id.clone(),
            calendar_id: calendar.id.clone(),
            duration: 1000 * 60 * 30, // 30 minutes
            reminders: Vec::new(),
            busy: Some(true),
            recurrence: Some(recurrence),
            exdates: None,
            recurring_event_id: None,
            original_start_time: None,
            service_id: None,
            start_time: recurring_event_start,
            metadata: None,
        })
        .await
        .unwrap()
        .event;

    // Test the iCal export endpoint using the SDK client (using default timespan)
    let ical_content = admin_client
        .calendar
        .export_ical(nittei_sdk::ExportCalendarIcalInput {
            calendar_id: calendar.id.clone(),
            start_time: None, // Use default (3 months ago)
            end_time: None,   // Use default (6 months in future)
        })
        .await
        .expect("Expected to export calendar as iCal");

    // Verify iCal content structure
    assert!(ical_content.contains("BEGIN:VCALENDAR"));
    assert!(ical_content.contains("END:VCALENDAR"));
    assert!(ical_content.contains("VERSION:2.0"));
    assert!(ical_content.contains("PRODID:-//Nittei//Calendar API//EN"));

    // Verify calendar properties
    assert!(ical_content.contains("X-WR-CALNAME:Recurring Events Calendar"));
    assert!(ical_content.contains("X-WR-TIMEZONE:UTC"));

    // Verify recurring event is present in iCal
    assert!(ical_content.contains("SUMMARY:Daily Meeting"));
    assert!(ical_content.contains("DESCRIPTION:Daily standup meeting"));
    assert!(ical_content.contains("LOCATION:Conference Room"));
    assert!(ical_content.contains("STATUS:CONFIRMED"));
    assert!(ical_content.contains("DTSTART:"));
    assert!(ical_content.contains("DTEND:"));

    // Verify recurrence rule is present
    assert!(ical_content.contains("RRULE:FREQ=DAILY;COUNT=5"));
}

#[tokio::test]
async fn test_export_empty_calendar_ical() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NitteiSDK::new(address.clone(), res.secret_api_key.clone());

    // Create user
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

    // Create calendar (no events)
    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::UTC,
            name: Some("Empty Calendar".to_string()),
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    // Test the iCal export endpoint using the SDK client (using default timespan)
    let ical_content = admin_client
        .calendar
        .export_ical(nittei_sdk::ExportCalendarIcalInput {
            calendar_id: calendar.id.clone(),
            start_time: None, // Use default (3 months ago)
            end_time: None,   // Use default (6 months in future)
        })
        .await
        .expect("Expected to export calendar as iCal");

    // Verify iCal content structure for empty calendar
    assert!(ical_content.contains("BEGIN:VCALENDAR"));
    assert!(ical_content.contains("END:VCALENDAR"));
    assert!(ical_content.contains("VERSION:2.0"));
    assert!(ical_content.contains("PRODID:-//Nittei//Calendar API//EN"));

    // Verify calendar properties
    assert!(ical_content.contains("X-WR-CALNAME:Empty Calendar"));
    assert!(ical_content.contains("X-WR-TIMEZONE:UTC"));

    // Verify no events are present (only calendar header/footer)
    assert!(!ical_content.contains("BEGIN:VEVENT"));
    assert!(!ical_content.contains("END:VEVENT"));
}

#[tokio::test]
async fn test_export_calendar_ical_unauthorized() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NitteiSDK::new(address.clone(), res.secret_api_key.clone());

    // Create user
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

    // Create calendar
    let calendar = admin_client
        .calendar
        .create(CreateCalendarInput {
            user_id: user.id.clone(),
            timezone: chrono_tz::UTC,
            name: Some("Test Calendar".to_string()),
            key: None,
            week_start: Weekday::Mon,
            metadata: None,
        })
        .await
        .unwrap()
        .calendar;

    // Test that unauthorized access fails by creating a new SDK client without API key
    let unauthorized_client = NitteiSDK::new(address, "invalid_api_key".to_string());

    // Try to export calendar as iCal with invalid API key - should fail
    let ical_result = unauthorized_client
        .calendar
        .export_ical(nittei_sdk::ExportCalendarIcalInput {
            calendar_id: calendar.id.clone(),
            start_time: None, // Use default (3 months ago)
            end_time: None,   // Use default (6 months in future)
        })
        .await;

    // Should return an error due to invalid API key
    assert!(ical_result.is_err());
}

#[tokio::test]
async fn test_export_calendar_ical_not_found() {
    let (app, sdk, address) = spawn_app().await;
    let res = sdk
        .account
        .create(&app.config.create_account_secret_code)
        .await
        .expect("Expected to create account");

    let admin_client = NitteiSDK::new(address.clone(), res.secret_api_key.clone());

    // Test the iCal export endpoint with non-existent calendar
    let non_existent_id = nittei_sdk::ID::default();

    // Try to export non-existent calendar as iCal - should fail
    let ical_result = admin_client
        .calendar
        .export_ical(nittei_sdk::ExportCalendarIcalInput {
            calendar_id: non_existent_id.clone(),
            start_time: None, // Use default (3 months ago)
            end_time: None,   // Use default (6 months in future)
        })
        .await;
    assert!(ical_result.is_err());
}
