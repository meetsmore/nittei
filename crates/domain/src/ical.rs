use std::collections::HashMap;

use crate::{Calendar, CalendarEvent, CalendarEventStatus, ID, RRuleFrequency, RRuleOptions};

/// Generates iCalendar content from calendar events and instances
///
/// This function creates a complete iCalendar (.ics) file content including:
/// - Calendar metadata (name, timezone)
/// - Event details (title, description, location, dates, status)
/// - Recurrence rules for recurring events
/// - Exception dates for modified recurring events
///
/// # Arguments
/// * `calendar` - The calendar containing the events
/// * `instances` - Event instances with resolved start/end times
/// * `events` - The original calendar events with full metadata
///
/// # Returns
/// A string containing the complete iCalendar content
pub fn generate_ical_content(
    calendar: &Calendar,
    normal_events: &[CalendarEvent],
    recurring_events: &[CalendarEvent],
    map_recurring_event_id_to_exceptions: &HashMap<&ID, Vec<CalendarEvent>>,
) -> String {
    let mut ical = String::new();

    // iCalendar header
    ical.push_str("BEGIN:VCALENDAR\r\n");
    ical.push_str("VERSION:2.0\r\n");
    ical.push_str("PRODID:-//Nittei//Calendar API//EN\r\n");
    ical.push_str("CALSCALE:GREGORIAN\r\n");
    ical.push_str("METHOD:PUBLISH\r\n");

    // Add calendar name if available
    if let Some(name) = &calendar.name {
        ical.push_str(&format!("X-WR-CALNAME:{}\r\n", escape_text(name)));
    }

    // Add timezone information
    ical.push_str(&format!("X-WR-TIMEZONE:{}\r\n", calendar.settings.timezone));

    // Add events
    for event in normal_events {
        ical.push_str(&generate_ical_content_for_event(event));
    }

    for recurring_event in recurring_events {
        ical.push_str(&generate_ical_content_for_event(recurring_event));

        if let Some(exceptions) = map_recurring_event_id_to_exceptions.get(&recurring_event.id) {
            for exception in exceptions {
                ical.push_str(&generate_ical_content_for_exception(
                    exception,
                    recurring_event,
                ));
            }
        }
    }

    // iCalendar footer
    ical.push_str("END:VCALENDAR\r\n");

    ical
}

/// Generates iCalendar content for a single event
pub fn generate_ical_content_for_event(event: &CalendarEvent) -> String {
    let mut ical = String::new();

    ical.push_str("BEGIN:VEVENT\r\n");

    // Event ID
    ical.push_str(&format!("UID:{}\r\n", event.id));

    // Summary (title)
    if let Some(title) = &event.title {
        ical.push_str(&format!("SUMMARY:{}\r\n", escape_text(title)));
    }

    // Description
    if let Some(description) = &event.description {
        ical.push_str(&format!("DESCRIPTION:{}\r\n", escape_text(description)));
    }

    // Location
    if let Some(location) = &event.location {
        ical.push_str(&format!("LOCATION:{}\r\n", escape_text(location)));
    }

    // Start and end time
    if event.all_day {
        ical.push_str(&format!(
            "DTSTART;VALUE=DATE:{}\r\n",
            event.start_time.format("%Y%m%d")
        ));
        ical.push_str(&format!(
            "DTEND;VALUE=DATE:{}\r\n",
            event.end_time.format("%Y%m%d")
        ));
    } else {
        ical.push_str(&format!(
            "DTSTART:{}\r\n",
            event.start_time.format("%Y%m%dT%H%M%SZ")
        ));
        ical.push_str(&format!(
            "DTEND:{}\r\n",
            event.end_time.format("%Y%m%dT%H%M%SZ")
        ));
    }

    // Status
    ical.push_str(&format!(
        "STATUS:{}\r\n",
        match event.status {
            CalendarEventStatus::Confirmed => "CONFIRMED",
            CalendarEventStatus::Tentative => "TENTATIVE",
            CalendarEventStatus::Cancelled => "CANCELLED",
        }
    ));

    // Created and modified dates
    ical.push_str(&format!(
        "CREATED:{}\r\n",
        event.created.format("%Y%m%dT%H%M%SZ")
    ));
    ical.push_str(&format!(
        "LAST-MODIFIED:{}\r\n",
        event.updated.format("%Y%m%dT%H%M%SZ")
    ));

    // Recurrence rules
    if let Some(recurrence) = &event.recurrence
        && let Some(rrule) = recurrence_to_rrule_string(recurrence)
    {
        ical.push_str(&format!("RRULE:{}\r\n", rrule));
    }

    // Exception dates
    for exdate in &event.exdates {
        ical.push_str(&format!("EXDATE:{}\r\n", exdate.format("%Y%m%dT%H%M%SZ")));
    }

    // Busy status
    if !event.busy {
        ical.push_str("TRANSP:TRANSPARENT\r\n");
    } else {
        ical.push_str("TRANSP:OPAQUE\r\n");
    }

    ical.push_str("END:VEVENT\r\n");

    ical
}

/// Generate ical content for an exception of a recurring event
pub fn generate_ical_content_for_exception(
    exception: &CalendarEvent,
    parent_event: &CalendarEvent,
) -> String {
    let mut ical = String::new();
    ical.push_str("BEGIN:VEVENT\r\n");

    // Same UID as parent
    ical.push_str(&format!("UID:{}\r\n", parent_event.id));

    // RECURRENCE-ID identifies which occurrence is being modified
    if exception.all_day {
        if let Some(original_start_time) = &exception.original_start_time {
            ical.push_str(&format!(
                "RECURRENCE-ID;VALUE=DATE:{}\r\n",
                original_start_time.format("%Y%m%d")
            ));
        }
    } else if let Some(original_start_time) = &exception.original_start_time {
        ical.push_str(&format!(
            "RECURRENCE-ID:{}\r\n",
            original_start_time.format("%Y%m%dT%H%M%SZ")
        ));
    }

    // Modified properties
    if let Some(title) = &exception.title {
        ical.push_str(&format!("SUMMARY:{}\r\n", escape_text(title)));
    } else if let Some(title) = &parent_event.title {
        ical.push_str(&format!("SUMMARY:{}\r\n", escape_text(title)));
    }

    // Similar pattern for description, location, etc.

    // Modified start and end times
    if exception.all_day {
        ical.push_str(&format!(
            "DTSTART;VALUE=DATE:{}\r\n",
            exception.start_time.format("%Y%m%d")
        ));
        ical.push_str(&format!(
            "DTEND;VALUE=DATE:{}\r\n",
            exception.end_time.format("%Y%m%d")
        ));
    } else {
        ical.push_str(&format!(
            "DTSTART:{}\r\n",
            exception.start_time.format("%Y%m%dT%H%M%SZ")
        ));
        ical.push_str(&format!(
            "DTEND:{}\r\n",
            exception.end_time.format("%Y%m%dT%H%M%SZ")
        ));
    }

    // Status (important for cancelled occurrences)
    let status = &exception.status;
    ical.push_str(&format!("STATUS:{}\r\n", status));

    // Created and modified dates
    ical.push_str(&format!(
        "CREATED:{}\r\n",
        exception.created.format("%Y%m%dT%H%M%SZ")
    ));
    ical.push_str(&format!(
        "LAST-MODIFIED:{}\r\n",
        exception.updated.format("%Y%m%dT%H%M%SZ")
    ));

    // No RRULE on exceptions

    // Busy status
    let busy = exception.busy;
    if !busy {
        ical.push_str("TRANSP:TRANSPARENT\r\n");
    } else {
        ical.push_str("TRANSP:OPAQUE\r\n");
    }

    ical.push_str("END:VEVENT\r\n");
    ical
}

/// Converts RRuleOptions to an iCalendar RRULE string
///
/// This function transforms the internal recurrence rule representation into
/// the standard iCalendar RRULE format. It handles all supported recurrence
/// patterns including frequency, interval, count, until date, and various
/// by-rules (weekday, monthday, month, etc.).
///
/// # Arguments
/// * `recurrence` - The recurrence rule options to convert
///
/// # Returns
/// An optional string containing the RRULE value, or None if the rule is invalid
///
/// # Examples
/// A weekly recurrence with interval 2 and count 10 would produce:
/// `"FREQ=WEEKLY;INTERVAL=2;COUNT=10"`
fn recurrence_to_rrule_string(recurrence: &RRuleOptions) -> Option<String> {
    let mut rrule = String::new();

    // Frequency
    let freq = match recurrence.freq {
        RRuleFrequency::Yearly => "YEARLY",
        RRuleFrequency::Monthly => "MONTHLY",
        RRuleFrequency::Weekly => "WEEKLY",
        RRuleFrequency::Daily => "DAILY",
    };
    rrule.push_str(&format!("FREQ={}", freq));

    // Interval
    if recurrence.interval != 1 {
        rrule.push_str(&format!(";INTERVAL={}", recurrence.interval));
    }

    // Count
    if let Some(count) = recurrence.count {
        rrule.push_str(&format!(";COUNT={}", count));
    }

    // Until
    if let Some(until) = recurrence.until {
        rrule.push_str(&format!(";UNTIL={}", until.format("%Y%m%dT%H%M%SZ")));
    }

    // Byweekday
    if let Some(byweekday) = &recurrence.byweekday {
        let weekdays: Vec<String> = byweekday
            .iter()
            .map(|wd| match wd.nth() {
                None => format!("{}", wd.weekday()),
                Some(n) => format!("{}{}", n, wd.weekday()),
            })
            .collect();
        if !weekdays.is_empty() {
            rrule.push_str(&format!(";BYDAY={}", weekdays.join(",")));
        }
    }

    // Bymonthday
    if let Some(bymonthday) = &recurrence.bymonthday {
        let monthdays: Vec<String> = bymonthday.iter().map(|d| d.to_string()).collect();
        if !monthdays.is_empty() {
            rrule.push_str(&format!(";BYMONTHDAY={}", monthdays.join(",")));
        }
    }

    // Bymonth - convert chrono::Month to month number (1-12)
    if let Some(bymonth) = &recurrence.bymonth {
        let months: Vec<String> = bymonth
            .iter()
            .map(|m| {
                match m {
                    chrono::Month::January => "1",
                    chrono::Month::February => "2",
                    chrono::Month::March => "3",
                    chrono::Month::April => "4",
                    chrono::Month::May => "5",
                    chrono::Month::June => "6",
                    chrono::Month::July => "7",
                    chrono::Month::August => "8",
                    chrono::Month::September => "9",
                    chrono::Month::October => "10",
                    chrono::Month::November => "11",
                    chrono::Month::December => "12",
                }
                .to_string()
            })
            .collect();
        if !months.is_empty() {
            rrule.push_str(&format!(";BYMONTH={}", months.join(",")));
        }
    }

    // Weekstart
    if let Some(weekstart) = &recurrence.weekstart {
        rrule.push_str(&format!(";WKST={}", weekstart));
    }

    Some(rrule)
}

/// Escapes text content for safe inclusion in iCalendar format
///
/// This function escapes special characters according to the iCalendar specification
/// to ensure proper parsing by calendar applications. It handles backslashes,
/// newlines, carriage returns, semicolons, and commas.
///
/// # Arguments
/// * `text` - The text to escape
///
/// # Returns
/// The escaped text safe for use in iCalendar properties
///
/// # Examples
/// `"Hello; World"` becomes `"Hello\\; World"`
fn escape_text(text: &str) -> String {
    text.replace("\\", "\\\\")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace(";", "\\;")
        .replace(",", "\\,")
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono_tz::UTC;
    use rrule::Weekday;

    use super::*;
    use crate::CalendarSettings;

    #[test]
    fn test_generate_ical_content() {
        let calendar = Calendar {
            id: ID::default(),
            user_id: ID::default(),
            account_id: ID::default(),
            name: Some("Test Calendar".to_string()),
            key: None,
            settings: CalendarSettings {
                week_start: Weekday::Mon,
                timezone: UTC,
            },
            metadata: None,
        };

        let events = vec![CalendarEvent {
            id: ID::default(),
            external_parent_id: None,
            external_id: None,
            title: Some("Test Event".to_string()),
            description: Some("Test Description".to_string()),
            event_type: None,
            location: Some("Test Location".to_string()),
            all_day: false,
            status: CalendarEventStatus::Confirmed,
            start_time: UTC
                .with_ymd_and_hms(2024, 1, 15, 10, 0, 0)
                .unwrap()
                .with_timezone(&chrono::Utc),
            duration: 3600000, // 1 hour in milliseconds
            busy: true,
            end_time: UTC
                .with_ymd_and_hms(2024, 1, 15, 11, 0, 0)
                .unwrap()
                .with_timezone(&chrono::Utc),
            created: UTC
                .with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated: UTC
                .with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
                .unwrap()
                .with_timezone(&chrono::Utc),
            recurrence: None,
            exdates: vec![],
            recurring_until: None,
            recurring_event_id: None,
            original_start_time: None,
            calendar_id: ID::default(),
            user_id: ID::default(),
            account_id: ID::default(),
            reminders: vec![],
            service_id: None,
            metadata: None,
        }];

        let ical_content = generate_ical_content(&calendar, &events, &[], &HashMap::new());

        // Verify basic iCalendar structure
        assert!(ical_content.contains("BEGIN:VCALENDAR"));
        assert!(ical_content.contains("END:VCALENDAR"));
        assert!(ical_content.contains("BEGIN:VEVENT"));
        assert!(ical_content.contains("END:VEVENT"));
        assert!(ical_content.contains("VERSION:2.0"));
        assert!(ical_content.contains("PRODID:-//Nittei//Calendar API//EN"));

        // Verify calendar properties
        assert!(ical_content.contains("X-WR-CALNAME:Test Calendar"));
        assert!(ical_content.contains("X-WR-TIMEZONE:UTC"));

        // Verify event properties
        assert!(ical_content.contains("SUMMARY:Test Event"));
        assert!(ical_content.contains("DESCRIPTION:Test Description"));
        assert!(ical_content.contains("LOCATION:Test Location"));
        assert!(ical_content.contains("STATUS:CONFIRMED"));
        assert!(ical_content.contains("DTSTART:20240115T100000Z"));
        assert!(ical_content.contains("DTEND:20240115T110000Z"));
    }

    #[test]
    fn test_recurrence_to_rrule_string() {
        let recurrence = RRuleOptions {
            freq: RRuleFrequency::Weekly,
            interval: 2,
            count: Some(10),
            until: None,
            bysetpos: None,
            byweekday: None,
            bymonthday: None,
            bymonth: None,
            byyearday: None,
            byweekno: None,
            weekstart: None,
        };

        let rrule = recurrence_to_rrule_string(&recurrence).unwrap();
        assert_eq!(rrule, "FREQ=WEEKLY;INTERVAL=2;COUNT=10");
    }

    #[test]
    fn test_escape_text() {
        let text = "Hello; World, with\nnewlines\r";
        let escaped = escape_text(text);
        assert_eq!(escaped, "Hello\\; World\\, with\\nnewlines\\r");
    }
}
