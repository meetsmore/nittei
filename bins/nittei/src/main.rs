mod telemetry;

use nittei_api::Application;
use nittei_infra::setup_context;
use telemetry::init_subscriber;
use tikv_jemallocator::Jemalloc;
use tokio::signal;
use tracing::info;

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
        let ctrl_c = async {
            #[allow(clippy::expect_used)]
            signal::ctrl_c()
                .await
                .expect("failed to install signal handler (ctrl_c)");
        };

        let terminate = async {
            #[allow(clippy::expect_used)]
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler (terminate)")
                .recv()
                .await;
        };

        tokio::select! {
            _ = ctrl_c => info!("[main_shutdown_handler] Received SIGINT"),
            _ = terminate => info!("[main_shutdown_handler] Received SIGTERM"),
        }
        let _ = tx.send(());
    });

    // Start the application and block until it finishes
    app.start(rx).await?;

    info!("[main_shutdown_handler] shutdown complete");

    Ok(())
}
