mod telemetry;

use nettu_scheduler_api::Application;
use nettu_scheduler_infra::setup_context;
use telemetry::init_subscriber;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    openssl_probe::init_ssl_cert_env_vars();

    init_subscriber();

    let context = setup_context().await;

    let app = Application::new(context).await?;
    app.start().await
}
