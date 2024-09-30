mod telemetry;

use nittei_api::Application;
use nittei_infra::setup_context;
use telemetry::init_subscriber;
// Use Jemalloc only for musl-64 bits platforms
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
use tikv_jemallocator::Jemalloc;

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

    let app = Application::new(context).await?;
    app.start().await
}
