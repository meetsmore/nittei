use nittei::telemetry::init_subscriber;
use nittei_infra::run_migration;
use tracing::{error, info};

/// This is a standalone binary that can be run to apply the migrations
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the subscriber for logging & tracing
    init_subscriber()?;

    run_migration().await.inspect_err(|e| {
        error!("Failed to run migrations: {}", e);
    })?;

    info!("Migrations complete");

    Ok(())
}
