use std::collections::HashMap;

use anyhow::anyhow;
use axum::{
    Extension,
    extract::{Path, Query},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Months, Utc};
use nittei_api_structs::get_calendar_events_ical::{PathParams, QueryParams};
use nittei_domain::{
    Account,
    ID,
    TimeSpan,
    User,
    expand_event_and_remove_exceptions,
    generate_ical_content,
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

/// Notes
/// - Right now it doesn't fetch all the ongoing recurring events (even though it should)
/// - It doesn't limit the number of events to export - ideally it should
/// - The endpoints are not public - ideally users should be able to expose the calendars to the public via a public link (but hard to find)

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
        ("start_time" = Option<DateTime<Utc>>, Query, description = "The start time of the events to export"),
        ("end_time" = Option<DateTime<Utc>>, Query, description = "The end time of the events to export"),
        ("limit" = Option<usize>, Query, description = "The limit for the number of events to export"),
        ("offset" = Option<usize>, Query, description = "The offset for the events to export"),
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
        ("start_time" = Option<DateTime<Utc>>, Query, description = "The start time of the events to export"),
        ("end_time" = Option<DateTime<Utc>>, Query, description = "The end time of the events to export"),
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
    pub start_time: Option<DateTime<Utc>>,
    /// The end time for the export range (UTC)
    pub end_time: Option<DateTime<Utc>>,
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

        // Create timespan for the export
        let timespan = TimeSpan::new(
            self.start_time.unwrap_or(
                Utc::now()
                    .checked_sub_months(Months::new(3))
                    .ok_or(anyhow!("Invalid start time"))
                    .map_err(|err| {
                        error!(
                            "[export_calendar_ical] Got an error while getting the start time: {:?}",
                            err
                        );
                        UseCaseError::InternalError
                    })?,
            ),
            self.end_time.unwrap_or(
                Utc::now()
                    .checked_add_months(Months::new(6))
                    .ok_or(anyhow!("Invalid end time"))
                    .map_err(|err| {
                        error!(
                            "[export_calendar_ical] Got an error while getting the end time: {:?}",
                            err
                        );
                        UseCaseError::InternalError
                    })?,
            ),
        );

        // Get events for the calendar in the specified time range
        let events = ctx
            .repos
            .events
            .find_by_calendar(&self.calendar_id, Some(timespan.clone()))
            .await
            .map_err(|err| {
                error!(
                    "[export_calendar_ical] Got an error while getting the events: {:?}",
                    err
                );
                UseCaseError::InternalError
            })?;

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
        let mut map_event_id_to_instances = HashMap::new();

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

            map_event_id_to_instances.insert(event.id.clone(), instances);
        }

        // Generate iCalendar content
        let ical_content = generate_ical_content(
            &calendar,
            &normal_events,
            &recurring_events,
            &map_event_id_to_instances,
        );

        Ok(UseCaseResponse { ical_content })
    }
}
