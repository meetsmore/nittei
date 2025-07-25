use chrono::{TimeDelta, Utc};
use futures::future;
use nittei_domain::{Calendar, CalendarEvent, EventRemindersExpansionJob, Reminder};
use nittei_infra::NitteiContext;
use tracing::error;

use crate::shared::usecase::UseCase;

#[derive(Debug)]
pub enum EventOperation {
    Created,
    Updated,
}

/// Synchronizes the upcoming `Reminders` for a `CalendarEvent`
#[derive(Debug)]
pub struct SyncEventRemindersUseCase<'a> {
    pub request: SyncEventRemindersTrigger<'a>,
}

#[derive(Debug)]
pub enum SyncEventRemindersTrigger<'a> {
    /// A `CalendarEvent` has been modified, e.g. deleted, updated og created.
    EventModified(&'a CalendarEvent, EventOperation),
    /// Periodic Job Scheduler that triggers this use case to perform
    /// `EventRemindersExpansionJob`s.
    JobScheduler,
}

#[derive(Debug)]
pub enum UseCaseError {
    StorageError,
    CalendarNotFound,
}

async fn create_event_reminders(
    event: &CalendarEvent,
    calendar: &Calendar,
    version: i64,
    ctx: &NitteiContext,
) -> Result<(), UseCaseError> {
    let timestamp_now_millis = ctx.sys.get_timestamp();
    let threshold_millis = timestamp_now_millis + TimeDelta::milliseconds(61 * 1000); // Now + 61 seconds

    let rrule_set = event.get_rrule_set(&calendar.settings).map_err(|e| {
        error!(
            event_id = %event.id,
            error = %e,
            "Unable to parse rrule set for event"
        );
        UseCaseError::StorageError
    })?;

    let reminders: Vec<Reminder> = match rrule_set {
        Some(rrule_set) => {
            let rrule_set_iter = rrule_set.into_iter();

            let max_delta_millis = event
                .reminders
                .iter()
                .max_by_key(|r| r.delta)
                .map(|r| r.delta * 60 * 1000)
                .unwrap_or(0);

            let mut future_occurrences_selected = 0;
            let now = ctx.sys.get_timestamp_millis();
            let dates = rrule_set_iter
                // Ignore occurrences of event that does not have a reminder in the future
                .skip_while(|d| d.timestamp_millis() + max_delta_millis < now)
                // Take the next 100 occurrences
                // .take(100)
                .take_while(|d| {
                    if d.timestamp_millis() >= now {
                        future_occurrences_selected += 1;
                        future_occurrences_selected <= 100
                    } else {
                        // This is possible if there are old occurrences with reminders still in the future
                        true
                    }
                })
                .collect::<Vec<_>>();

            if dates.len() == 100 {
                // There are more reminders to generate, store a job to expand them later
                let job = EventRemindersExpansionJob {
                    event_id: event.id.clone(),
                    timestamp: dates[90].with_timezone(&Utc),
                    version,
                };
                if ctx
                    .repos
                    .event_reminders_generation_jobs
                    .bulk_insert(&[job])
                    .await
                    .is_err()
                {
                    error!(
                        event_id = %event.id,
                        "Unable to store event reminders expansion job for event"
                    );
                }
            }

            dates
                .into_iter()
                .flat_map(|d| {
                    let dt_millis = d;
                    event
                        .reminders
                        .iter()
                        .map(|er| {
                            let delta_millis = er.delta * 60 * 1000;
                            let remind_at = dt_millis + TimeDelta::milliseconds(delta_millis);
                            (er, remind_at)
                        })
                        .filter(|(_er, remind_at)| remind_at > &threshold_millis)
                        .map(|(er, remind_at)| Reminder {
                            event_id: event.id.to_owned(),
                            account_id: event.account_id.to_owned(),
                            remind_at: remind_at.with_timezone(&Utc),
                            version,
                            identifier: er.identifier.clone(),
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        }
        None => event
            .reminders
            .iter()
            .map(|er| {
                let delta_millis = er.delta * 60 * 1000;
                let remind_at = event.start_time + TimeDelta::milliseconds(delta_millis);
                (er, remind_at)
            })
            .filter(|(_er, remind_at)| remind_at > &threshold_millis)
            .map(|(er, remind_at)| Reminder {
                event_id: event.id.to_owned(),
                account_id: event.account_id.to_owned(),
                remind_at,
                version,
                identifier: er.identifier.clone(),
            })
            .collect(),
    };

    // create reminders for the next `self.expansion_interval`
    ctx.repos
        .reminders
        .bulk_insert(&reminders)
        .await
        .map_err(|_| UseCaseError::StorageError)?;

    Ok(())
}

#[async_trait::async_trait]
impl UseCase for SyncEventRemindersUseCase<'_> {
    type Response = ();

    type Error = UseCaseError;

    const NAME: &'static str = "SyncEventReminders";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        match &self.request {
            SyncEventRemindersTrigger::EventModified(calendar_event, op) => {
                let version = match op {
                    EventOperation::Created => ctx
                        .repos
                        .reminders
                        .init_version(&calendar_event.id)
                        .await
                        .map_err(|e| {
                            error!(
                                event_id = %calendar_event.id,
                                error = %e,
                                "Unable to init event reminder version"
                            );
                            UseCaseError::StorageError
                        })?,
                    // Delete existing reminders
                    &EventOperation::Updated => ctx
                        .repos
                        .reminders
                        .inc_version(&calendar_event.id)
                        .await
                        .map_err(|e| {
                            error!(
                                event_id = %calendar_event.id,
                                error = %e,
                                "Unable to increment event reminder version"
                            );
                            UseCaseError::StorageError
                        })?,
                };

                // Create new reminders
                let calendar = ctx
                    .repos
                    .calendars
                    .find(&calendar_event.calendar_id)
                    .await
                    .map_err(|_| UseCaseError::StorageError)?
                    .ok_or(UseCaseError::CalendarNotFound)?;

                create_event_reminders(calendar_event, &calendar, version, ctx)
                    .await
                    .map_err(|e| {
                        error!(
                            event_id = %calendar_event.id,
                            error = ?e,
                            "Unable to create event reminders for event",
                        );
                        e
                    })
            }
            SyncEventRemindersTrigger::JobScheduler => {
                let jobs = ctx
                    .repos
                    .event_reminders_generation_jobs
                    .delete_all_before(ctx.sys.get_timestamp())
                    .await
                    .map_err(|_| UseCaseError::StorageError)?;

                let event_ids = jobs
                    .iter()
                    .map(|job| job.event_id.to_owned())
                    .collect::<Vec<_>>();

                let events = match ctx.repos.events.find_many(&event_ids).await {
                    Ok(events) => events,
                    Err(_) => return Err(UseCaseError::StorageError),
                };

                future::join_all(
                    events
                        .into_iter()
                        .map(|event| generate_event_reminders_job(event, ctx))
                        .collect::<Vec<_>>(),
                )
                .await;

                Ok(())
            }
        }
    }
}

async fn generate_event_reminders_job(event: CalendarEvent, ctx: &NitteiContext) {
    let calendar = match ctx.repos.calendars.find(&event.calendar_id).await {
        Ok(Some(calendar)) => calendar,
        Ok(None) => {
            error!(
                event_id = %event.id,
                calendar_id = %event.calendar_id,
                "Unable to find calendar for event reminders job",
            );
            return;
        }
        Err(e) => {
            error!(
                event_id = %event.id,
                calendar_id = %event.calendar_id,
                error = %e,
                "Unable to find calendar for event reminders job",
            );
            return;
        }
    };

    let version = match ctx.repos.reminders.inc_version(&event.id).await {
        Ok(version) => version,
        Err(e) => {
            error!(
                event_id = %event.id,
                error = %e,
                "Unable to increment event reminder version for job",
            );
            return;
        }
    };

    if let Err(e) = create_event_reminders(&event, &calendar, version, ctx).await {
        error!(
            event_id = %event.id,
            error = ?e,
            "Unable to create event reminders for job",
        );
    }
}
