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

use std::{sync::Arc, time::Duration};

use account::{IAccountRepo, PostgresAccountRepo};
use account_integrations::{IAccountIntegrationRepo, PostgresAccountIntegrationRepo};
use anyhow::Context;
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
pub use event::{SearchEventsForAccountParams, SearchEventsForUserParams, SearchEventsParams};
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
use sqlx::{Pool, Postgres, migrate::MigrateError, postgres::PgPoolOptions};
use status::{IStatusRepo, PostgresStatusRepo};
use tracing::{error, info};
use user::{IUserRepo, PostgresUserRepo};
use user_integrations::{IUserIntegrationRepo, PostgresUserIntegrationRepo};

use crate::metrics::{register_metrics, update_connection_pool_metrics};

/// Wrapper around PgPool that tracks connection pool metrics
#[derive(Clone)]
pub struct MonitoredPgPool {
    pool: Pool<Postgres>,
}

impl MonitoredPgPool {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn update_metrics(&self) {
        let total = self.pool.size() as i64;
        let idle = self.pool.num_idle() as i64;
        let busy = total - idle;

        update_connection_pool_metrics(total, idle, busy);
    }
}

impl std::ops::Deref for MonitoredPgPool {
    type Target = Pool<Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

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
        // Register metrics
        register_metrics()?;

        info!("[repos] Creating postgres connection");
        let pool = PgPoolOptions::new()
            .min_connections(nittei_utils::config::APP_CONFIG.pg.min_connections)
            .max_connections(nittei_utils::config::APP_CONFIG.pg.max_connections)
            .acquire_timeout(Duration::from_secs(3)) // Max time to wait for a connection
            .connect(connection_string)
            .await
            .context(format!(
                "Failed to connect to PG url '{}'",
                remove_password_from_url(connection_string)?
            ))?;

        info!("[repos] Postgres connection created");

        // Create monitored pool
        let monitored_pool = MonitoredPgPool::new(pool.clone());

        if !nittei_utils::config::APP_CONFIG.pg.skip_migrations {
            info!("[repos] Executing migrations");

            // Run the migrations
            let migration_result = sqlx::migrate!().run(&pool).await;

            // Check if the migration failed
            // If the migration failed because the migration was previously applied but is missing in the resolved migrations, log the error and continue
            // This can happen if the migration was applied by a new deployment, but the app itself failed to start completely
            // In order to avoid breaking the old deployment (potentially restarting), we log the error and continue
            if let Err(e) = migration_result {
                if let MigrateError::VersionMissing(_) = e {
                    error!("Failed to run migration: {}", e);
                    // Log the error and do not propagate it
                } else {
                    // Return early the error
                    return Err(e.into());
                }
            }
            info!("[repos] Migrations executed");
        } else {
            info!("[repos] Migrations skipped");
        }

        // Start background task to update metrics
        let monitored_pool_clone = monitored_pool.clone();
        tokio::spawn(async move {
            // Update metrics every 5 seconds
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                monitored_pool_clone.update_metrics().await;
            }
        });

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

fn remove_password_from_url(connection_string: &str) -> anyhow::Result<String> {
    let mut url = match url::Url::parse(connection_string) {
        Ok(url) => url,
        // If the connection string is not a valid URL, return the connection string as is
        Err(_) => return Ok(connection_string.to_string()),
    };
    #[allow(clippy::unwrap_used)]
    url.set_password(Some("*********")).unwrap();
    Ok(url.to_string())
}
