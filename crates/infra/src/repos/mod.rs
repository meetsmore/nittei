mod account;
mod account_integrations;
mod calendar;
mod calendar_synced;
mod event;
mod reservation;
mod schedule;
mod service;
mod service_user;
mod service_user_busy_calendars;
mod shared;
mod status;
pub(crate) mod user;
mod user_integrations;

use std::sync::Arc;

use account::{IAccountRepo, PostgresAccountRepo};
use account_integrations::{IAccountIntegrationRepo, PostgresAccountIntegrationRepo};
use calendar::{ICalendarRepo, PostgresCalendarRepo};
use calendar_synced::{ICalendarSyncedRepo, PostgresCalendarSyncedRepo};
use event::{
    IEventRemindersGenerationJobsRepo,
    IEventRepo,
    IEventSyncedRepo,
    IReminderRepo,
    PostgresEventReminderGenerationJobsRepo,
    PostgresEventRepo,
    PostgresEventSyncedRepo,
    PostgresReminderRepo,
};
use reservation::{IReservationRepo, PostgresReservationRepo};
use schedule::{IScheduleRepo, PostgresScheduleRepo};
use service::{IServiceRepo, PostgresServiceRepo};
use service_user::{IServiceUserRepo, PostgresServiceUserRepo};
pub use service_user_busy_calendars::{BusyCalendarIdentifier, ExternalBusyCalendarIdentifier};
use service_user_busy_calendars::{
    IServiceUserBusyCalendarRepo,
    PostgresServiceUseBusyCalendarRepo,
};
pub use shared::query_structs::*;
use sqlx::postgres::PgPoolOptions;
use status::{IStatusRepo, PostgresStatusRepo};
use tracing::{error, info};
use user::{IUserRepo, PostgresUserRepo};
use user_integrations::{IUserIntegrationRepo, PostgresUserIntegrationRepo};

#[derive(Clone)]
pub struct Repos {
    pub accounts: Arc<dyn IAccountRepo>,
    pub account_integrations: Arc<dyn IAccountIntegrationRepo>,
    pub calendars: Arc<dyn ICalendarRepo>,
    pub calendar_synced: Arc<dyn ICalendarSyncedRepo>,
    pub events: Arc<dyn IEventRepo>,
    pub event_reminders_generation_jobs: Arc<dyn IEventRemindersGenerationJobsRepo>,
    pub event_synced: Arc<dyn IEventSyncedRepo>,
    pub schedules: Arc<dyn IScheduleRepo>,
    pub reminders: Arc<dyn IReminderRepo>,
    pub reservations: Arc<dyn IReservationRepo>,
    pub services: Arc<dyn IServiceRepo>,
    pub service_users: Arc<dyn IServiceUserRepo>,
    pub service_user_busy_calendars: Arc<dyn IServiceUserBusyCalendarRepo>,
    pub status: Arc<dyn IStatusRepo>,
    pub users: Arc<dyn IUserRepo>,
    pub user_integrations: Arc<dyn IUserIntegrationRepo>,
}

impl Repos {
    pub async fn create_postgres(connection_string: &str) -> anyhow::Result<Self> {
        info!("DB CHECKING CONNECTION ...");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await?;
        info!("DB CHECKING CONNECTION ... [done]");

        if std::env::var("nittei_SKIP_MIGRATION").is_err() {
            info!("DB EXECUTING MIGRATION ...");

            // Run the migrations
            let migration_result = sqlx::migrate!().run(&pool).await;

            // Check if the migration failed
            // If the migration failed because the migration was previously applied but is missing in the resolved migrations, log the error and continue
            // This can happen if the migration was applied by a new deployment, but the app itself failed to start completely
            // In order to avoid breaking the old deployment (potentially restarting), we log the error and continue
            if let Err(e) = migration_result {
                // Convert error to string to check the message
                let error_message = e.to_string();

                if error_message
                    .contains("was previously applied but is missing in the resolved migrations")
                {
                    error!("Failed to run migration: {}", error_message);
                    // Continue, i.e., log the error and do not propagate it
                } else {
                    // Return early the error
                    return Err(e.into());
                }
            }
            info!("DB EXECUTING MIGRATION ... [done]");
        } else {
            info!("DB MIGRATION SKIPPED");
        }

        Ok(Self {
            accounts: Arc::new(PostgresAccountRepo::new(pool.clone())),
            account_integrations: Arc::new(PostgresAccountIntegrationRepo::new(pool.clone())),
            calendars: Arc::new(PostgresCalendarRepo::new(pool.clone())),
            calendar_synced: Arc::new(PostgresCalendarSyncedRepo::new(pool.clone())),
            events: Arc::new(PostgresEventRepo::new(pool.clone())),
            event_synced: Arc::new(PostgresEventSyncedRepo::new(pool.clone())),
            users: Arc::new(PostgresUserRepo::new(pool.clone())),
            user_integrations: Arc::new(PostgresUserIntegrationRepo::new(pool.clone())),
            services: Arc::new(PostgresServiceRepo::new(pool.clone())),
            service_users: Arc::new(PostgresServiceUserRepo::new(pool.clone())),
            service_user_busy_calendars: Arc::new(PostgresServiceUseBusyCalendarRepo::new(
                pool.clone(),
            )),
            schedules: Arc::new(PostgresScheduleRepo::new(pool.clone())),
            reminders: Arc::new(PostgresReminderRepo::new(pool.clone())),
            reservations: Arc::new(PostgresReservationRepo::new(pool.clone())),
            event_reminders_generation_jobs: Arc::new(
                PostgresEventReminderGenerationJobsRepo::new(pool.clone()),
            ),
            status: Arc::new(PostgresStatusRepo::new(pool)),
        })
    }
}
