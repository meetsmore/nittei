mod telemetry;

use nittei_api::Application;
use nittei_infra::setup_context;
use nittei_utils::config::APP_CONFIG;
use telemetry::init_subscriber;
use tikv_jemallocator::Jemalloc;
use tokio::{runtime::Builder, signal};
use tracing::{error, info};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> anyhow::Result<()> {
    // Initialize the subscriber for logging & tracing
    init_subscriber()?;

    // Read the environment variable (default to "multi_thread" if not set)
    let runtime_mode = &APP_CONFIG.tokio_runtime;

    let runtime = if runtime_mode == "current_thread" {
        info!("Using single-threaded Tokio runtime.");
        Builder::new_current_thread().enable_all().build()?
    } else if runtime_mode == "multi_thread" {
        info!("Using multi-threaded Tokio runtime.");
        Builder::new_multi_thread().enable_all().build()?
    } else {
        error!(
            "Invalid value for `tokio_runtime` in the configuration: {} - defaulting to `multi_thread`",
            runtime_mode
        );
        Builder::new_multi_thread().enable_all().build()?
    };

    runtime.block_on(async_main())?;

    Ok(())
}

async fn async_main() -> anyhow::Result<()> {
    let context = setup_context().await?;
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let app = Application::new(context).await?;

    // Listen for SIGINT (Ctrl+C) to shutdown the service
    // This sends a message on the channel to shutdown the server gracefully
    // It waits for a configurable amount of seconds (in order for the pod to be removed from the k8s service)
    // And then waits for the server to finish processing the current requests before shutting down
    tokio::spawn(async move {
        if let Err(e) = signal::ctrl_c().await {
            error!("[main] Failed to listen for SIGINT: {}", e);
        }
        info!("[shutdown] Received SIGINT, sending event on channel...");
        let _ = tx.send(());
    });

    // Start the application and block until it finishes
    app.start(rx).await?;

    info!("[shutdown] shutdown complete");

    Ok(())
}
