mod telemetry;

use nettu_scheduler_api::Application;
use nettu_scheduler_infra::setup_context;
use telemetry::init_subscriber;

// Use Jemalloc only for musl-64 bits platforms
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize the environment variables for SSL certificates
    openssl_probe::init_ssl_cert_env_vars();

    // Initialize the subscriber for logging
    init_subscriber();

    let context = setup_context().await;

    let app = Application::new(context).await?;
    app.start().await
}
