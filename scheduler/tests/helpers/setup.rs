// Allow clippy lints because this is a test helper
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
use nettu_scheduler_api::Application;
use nettu_scheduler_infra::{setup_context, Config, NettuContext};
use nettu_scheduler_sdk::NettuSDK;

#[allow(dead_code)]
pub struct TestApp {
    pub config: Config,
    pub ctx: NettuContext,
}

// Launch the application as a background task
pub async fn spawn_app() -> (TestApp, NettuSDK, String) {
    let mut ctx = setup_context().await.unwrap();
    ctx.config.port = 0; // Random port

    let config = ctx.config.clone();
    let context = ctx.clone();
    let application = Application::new(ctx)
        .await
        .expect("Failed to build application.");

    let address = format!("http://localhost:{}", application.port());

    // Allow underscore future because it needs to run in background
    // If we `await` it, the tests will hang
    #[allow(clippy::let_underscore_future)]
    let _ = actix_web::rt::spawn(async move {
        application
            .start()
            .await
            .expect("Expected application to start");
    });

    let app = TestApp {
        config,
        ctx: context,
    };
    let sdk = NettuSDK::new(address.clone(), "");
    (app, sdk, address)
}
