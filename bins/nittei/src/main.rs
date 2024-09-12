mod telemetry;

use nittei_api::Application;
use nittei_infra::setup_context;
use telemetry::init_subscriber;

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
