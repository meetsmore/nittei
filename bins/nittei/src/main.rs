mod telemetry;

use nittei_api::Application;
use nittei_infra::setup_context;
use telemetry::init_subscriber;
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
use tikv_jemallocator::Jemalloc;
use tokio::signal;
use tracing::{error, info};

// Use Jemalloc only for musl-64 bits platforms
// The default MUSL allocator is known to be slower than Jemalloc
// E.g. https://github.com/BurntSushi/ripgrep/commit/03bf37ff4a29361c47843369f7d3dc5689b8fdac
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the environment variables for SSL certificates
    openssl_probe::init_ssl_cert_env_vars();

    // Initialize the subscriber for logging & tracing
    init_subscriber()?;

    let context = setup_context().await?;
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let app = Application::new(context).await?;

    // Listen for SIGINT (Ctrl+C) to shutdown the service
    // This sends a message on the channel to shutdown the server gracefully
    // By doing so, it makes the app return failed status on the status API endpoint (useful for k8s)
    // It waits for a configurable amount of seconds (in order for the readiness probe to fail)
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
