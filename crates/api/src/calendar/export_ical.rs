use axum::{
    Extension,
    extract::{Path, Query},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use nittei_api_structs::get_calendar_events::{PathParams, QueryParams};
use nittei_domain::{
    Account,
    Calendar,
    CalendarEvent,
    CalendarEventStatus,
    EventInstance,
    ID,
    RRuleFrequency,
    RRuleOptions,
    TimeSpan,
    User,
    expand_event_and_remove_exceptions,
    generate_map_exceptions_original_start_times,
};
use nittei_infra::NitteiContext;
use tracing::error;

use crate::{
    error::NitteiError,
    shared::{
        auth::{Policy, account_can_modify_calendar},
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/user/calendar/{calendar_id}/ical",
    summary = "Export calendar events as iCalendar format (admin only)",
    security(
        ("api_key" = [])
    ),
    params(
        ("calendar_id" = ID, Path, description = "The id of the calendar to export"),
        ("start_time" = DateTime<Utc>, Query, description = "The start time of the events to export"),
        ("end_time" = DateTime<Utc>, Query, description = "The end time of the events to export"),
    ),
    responses(
        (status = 200, description = "iCalendar file", content_type = "text/calendar")
    )
)]
/// Export calendar events as iCalendar format for admin users
///
/// This endpoint allows admin users to export events from any calendar as an iCalendar (.ics) file.
/// The exported file can be imported into any calendar application that supports the iCalendar format.
///
/// # Parameters
/// - `calendar_id`: The ID of the calendar to export
/// - `start_time`: The start time for the export range (UTC)
/// - `end_time`: The end time for the export range (UTC)
///
/// # Returns
/// Returns an iCalendar file with Content-Type: text/calendar
pub async fn export_calendar_ical_admin_controller(
    Extension(account): Extension<Account>,
    query_params: Query<QueryParams>,
    path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Response, NitteiError> {
    let cal = account_can_modify_calendar(&account, &path.calendar_id, &ctx).await?;

    let usecase = ExportCalendarIcalUseCase {
        user_id: cal.user_id,
        calendar_id: cal.id,
        start_time: query_params.start_time,
        end_time: query_params.end_time,
    };

    execute(usecase, &ctx)
        .await
        .map_err(NitteiError::from)
        .map(|ical_content| {
            let headers = [
                (
                    "content-type",
                    HeaderValue::from_static("text/calendar; charset=utf-8"),
                ),
                (
                    "content-disposition",
                    HeaderValue::from_static("attachment; filename=\"calendar.ics\""),
                ),
            ];

            (StatusCode::OK, headers, ical_content.ical_content).into_response()
        })
}

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/calendar/{calendar_id}/ical",
    summary = "Export calendar events as iCalendar format",
    params(
        ("calendar_id" = ID, Path, description = "The id of the calendar to export"),
        ("start_time" = DateTime<Utc>, Query, description = "The start time of the events to export"),
        ("end_time" = DateTime<Utc>, Query, description = "The end time of the events to export"),
    ),
    responses(
        (status = 200, description = "iCalendar file", content_type = "text/calendar")
    )
)]
/// Export calendar events as iCalendar format for regular users
///
/// This endpoint allows users to export events from their own calendars as an iCalendar (.ics) file.
/// The exported file can be imported into any calendar application that supports the iCalendar format.
///
/// # Parameters
/// - `calendar_id`: The ID of the calendar to export (must belong to the authenticated user)
/// - `start_time`: The start time for the export range (UTC)
/// - `end_time`: The end time for the export range (UTC)
///
/// # Returns
/// Returns an iCalendar file with Content-Type: text/calendar
pub async fn export_calendar_ical_controller(
    Extension((user, _policy)): Extension<(User, Policy)>,
    query_params: Query<QueryParams>,
    path: Path<PathParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Response, NitteiError> {
    let usecase = ExportCalendarIcalUseCase {
        user_id: user.id,
        calendar_id: path.calendar_id.clone(),
        start_time: query_params.start_time,
        end_time: query_params.end_time,
    };

    execute(usecase, &ctx)
        .await
        .map_err(NitteiError::from)
        .map(|ical_content| {
            let headers = [
                (
                    "content-type",
                    HeaderValue::from_static("text/calendar; charset=utf-8"),
                ),
                (
                    "content-disposition",
                    HeaderValue::from_static("attachment; filename=\"calendar.ics\""),
                ),
            ];

            (StatusCode::OK, headers, ical_content.ical_content).into_response()
        })
}

/// Use case for exporting calendar events as iCalendar format
///
/// This use case handles the business logic for retrieving calendar events
/// within a specified time range and generating iCalendar content.
#[derive(Debug)]
pub struct ExportCalendarIcalUseCase {
    /// The ID of the calendar to export
    pub calendar_id: ID,
    /// The ID of the user who owns the calendar
    pub user_id: ID,
    /// The start time for the export range (UTC)
    pub start_time: DateTime<Utc>,
    /// The end time for the export range (UTC)
    pub end_time: DateTime<Utc>,
}

/// Response containing the generated iCalendar content
#[derive(Debug)]
pub struct UseCaseResponse {
    /// The generated iCalendar content as a string
    pub ical_content: String,
}

/// Errors that can occur during iCalendar export
#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    /// The requested calendar was not found or the user doesn't have access to it
    #[error("Calendar not found")]
    CalendarNotFound,
    /// An internal error occurred during processing
    #[error("Internal error")]
    InternalError,
}

impl From<UseCaseError> for NitteiError {
    fn from(error: UseCaseError) -> Self {
        match error {
            UseCaseError::CalendarNotFound => NitteiError::NotFound("Calendar".to_string()),
            UseCaseError::InternalError => NitteiError::InternalError,
        }
    }
}

#[async_trait::async_trait]
impl UseCase for ExportCalendarIcalUseCase {
    type Response = UseCaseResponse;
    type Error = UseCaseError;

    const NAME: &'static str = "ExportCalendarIcal";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        // Get the calendar
        let calendar = ctx
            .repos
            .calendars
            .find(&self.calendar_id)
            .await
            .map_err(|_| UseCaseError::InternalError)?
            .ok_or(UseCaseError::CalendarNotFound)?;

        // Verify the calendar belongs to the user
        if calendar.user_id != self.user_id {
            return Err(UseCaseError::CalendarNotFound);
        }

        // Get events for the calendar in the specified time range
        let timespan = TimeSpan::new(self.start_time, self.end_time);

        let events = ctx
            .repos
            .events
            .find_by_calendar(&self.calendar_id, Some(timespan.clone()))
            .await
            .map_err(|_| UseCaseError::InternalError)?;

        // Clone events to avoid ownership issues
        let events_clone = events.clone();

        // Separate normal events, recurring events, and exceptions
        let (normal_events, recurring_events, exceptions) = events.into_iter().fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut normal, mut recurring, mut exceptions), event| {
                if event.recurring_event_id.is_some() {
                    exceptions.push(event);
                } else if event.recurrence.is_some() {
                    recurring.push(event);
                } else {
                    normal.push(event);
                }
                (normal, recurring, exceptions)
            },
        );

        // Generate map of exceptions for recurring events
        let map_recurring_event_id_to_exceptions =
            generate_map_exceptions_original_start_times(&exceptions);

        // Expand recurring events and remove exceptions
        let mut all_instances = Vec::new();

        for event in &recurring_events {
            let exceptions = map_recurring_event_id_to_exceptions
                .get(&event.id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);

            let instances =
                expand_event_and_remove_exceptions(&calendar, event, exceptions, timespan.clone())
                    .map_err(|e| {
                        error!(
                            "[export_calendar_ical] Got an error while expanding an event: {:?}",
                            e
                        );
                        UseCaseError::InternalError
                    })?;

            all_instances.extend(instances);
        }

        // Add normal events as instances
        for event in &normal_events {
            if !exceptions
                .iter()
                .any(|e| e.original_start_time == Some(event.start_time))
            {
                all_instances.push(EventInstance {
                    start_time: event.start_time,
                    end_time: event.end_time,
                    busy: event.busy,
                });
            }
        }

        // Generate iCalendar content
        let ical_content = generate_ical_content(&calendar, &all_instances, &events_clone);

        Ok(UseCaseResponse { ical_content })
    }
}

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
fn generate_ical_content(
    calendar: &Calendar,
    instances: &[EventInstance],
    events: &[CalendarEvent],
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
    for event in events {
        if let Some(instance) = instances
            .iter()
            .find(|i| i.start_time == event.start_time && i.end_time == event.end_time)
        {
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
                    instance.start_time.format("%Y%m%d")
                ));
                ical.push_str(&format!(
                    "DTEND;VALUE=DATE:{}\r\n",
                    instance.end_time.format("%Y%m%d")
                ));
            } else {
                ical.push_str(&format!(
                    "DTSTART:{}\r\n",
                    instance.start_time.format("%Y%m%dT%H%M%SZ")
                ));
                ical.push_str(&format!(
                    "DTEND:{}\r\n",
                    instance.end_time.format("%Y%m%dT%H%M%SZ")
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
            }

            ical.push_str("END:VEVENT\r\n");
        }
    }

    // iCalendar footer
    ical.push_str("END:VCALENDAR\r\n");

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
    use nittei_domain::{CalendarSettings, RRuleFrequency, RRuleOptions, Weekday};

    use super::*;

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

        let instances = vec![EventInstance {
            start_time: UTC
                .with_ymd_and_hms(2024, 1, 15, 10, 0, 0)
                .unwrap()
                .with_timezone(&chrono::Utc),
            end_time: UTC
                .with_ymd_and_hms(2024, 1, 15, 11, 0, 0)
                .unwrap()
                .with_timezone(&chrono::Utc),
            busy: true,
        }];

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

        let ical_content = generate_ical_content(&calendar, &instances, &events);

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
