mod config;
pub mod metrics;
mod repos;
mod services;
mod system;

use std::sync::Arc;

pub use config::Config;
use repos::Repos;
pub use repos::{
    BusyCalendarIdentifier,
    ExternalBusyCalendarIdentifier,
    MetadataFindQuery,
    SearchEventsForAccountParams,
    SearchEventsForUserParams,
    SearchEventsParams,
};
pub use services::*;
pub use system::ISys;
use system::RealSys;

use crate::repos::create_postgres_pool;

/// The context for the application
/// Contains the repositories, configuration, and system
///
/// System is abstracted to allow for testing
#[derive(Clone)]
pub struct NitteiContext {
    pub repos: Repos,
    pub config: Config,
    pub sys: Arc<dyn ISys>,
}

/// The parameters to create the context
struct ContextParams {
    pub postgres_connection_string: String,
}

impl NitteiContext {
    async fn create(params: ContextParams) -> anyhow::Result<Self> {
        let repos = Repos::create_postgres(&params.postgres_connection_string).await?;
        Ok(Self {
            repos,
            config: Config::new(),
            sys: Arc::new(RealSys {}),
        })
    }
}

/// Will setup the infrastructure context given the environment
pub async fn setup_context() -> anyhow::Result<NitteiContext> {
    NitteiContext::create(ContextParams {
        postgres_connection_string: nittei_utils::config::APP_CONFIG.pg.database_url.clone(),
    })
    .await
}

/// Run the migrations
///
/// This is not run by the application itself, but is provided as a utility
/// Usage is in bins/nittei/src/bin/migrate.rs
pub async fn run_migration() -> anyhow::Result<()> {
    let pool =
        create_postgres_pool(nittei_utils::config::APP_CONFIG.pg.database_url.as_str()).await?;

    sqlx::migrate!().run(&pool).await.map_err(|e| e.into())
}
