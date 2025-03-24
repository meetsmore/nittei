use nittei::telemetry::init_subscriber;
use nittei_api::Application;
use nittei_infra::setup_context;
use tikv_jemallocator::Jemalloc;
use tokio::signal;
use tracing::{error, info};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the subscriber for logging & tracing
    init_subscriber()?;

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
