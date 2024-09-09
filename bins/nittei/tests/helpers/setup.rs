// Allow clippy lints because this is a test helper
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
use nittei_api::Application;
use nittei_infra::{setup_context, Config, NitteiContext};
use nittei_sdk::NitteiSDK;

#[allow(dead_code)]
pub struct TestApp {
    pub config: Config,
    pub ctx: NitteiContext,
}

// Launch the application as a background task
pub async fn spawn_app() -> (TestApp, NitteiSDK, String) {
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
    let sdk = NitteiSDK::new(address.clone(), "");
    (app, sdk, address)
}
