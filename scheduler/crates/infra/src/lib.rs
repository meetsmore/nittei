mod config;
mod repos;
mod services;
mod system;

use std::sync::Arc;

pub use config::Config;
use repos::Repos;
pub use repos::{BusyCalendarIdentifier, ExternalBusyCalendarIdentifier, MetadataFindQuery};
pub use services::*;
use sqlx::postgres::PgPoolOptions;
pub use system::ISys;
use system::RealSys;

#[derive(Clone)]
pub struct NettuContext {
    pub repos: Repos,
    pub config: Config,
    pub sys: Arc<dyn ISys>,
}

struct ContextParams {
    pub postgres_connection_string: String,
}

impl NettuContext {
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
pub async fn setup_context() -> anyhow::Result<NettuContext> {
    NettuContext::create(ContextParams {
        postgres_connection_string: get_psql_connection_string()?,
    })
    .await
}

fn get_psql_connection_string() -> anyhow::Result<String> {
    const PSQL_CONNECTION_STRING: &str = "DATABASE_URL";

    Ok(std::env::var(PSQL_CONNECTION_STRING)?)
}

pub async fn run_migration() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&get_psql_connection_string()?)
        .await?;

    sqlx::migrate!().run(&pool).await.map_err(|e| e.into())
}
