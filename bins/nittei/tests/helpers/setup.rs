use nittei_api::Application;
use nittei_infra::{Config, NitteiContext, setup_context};
use nittei_sdk::NitteiSDK;

#[allow(dead_code)]
pub struct TestApp {
    pub config: Config,
    pub ctx: NitteiContext,
}

#[cfg(test)]
// Launch the application as a background task
pub async fn spawn_app() -> (TestApp, NitteiSDK, String) {
    let mut ctx = setup_context().await.unwrap();
    ctx.config.port = 0; // Random port

    let config = ctx.config.clone();
    let context = ctx.clone();
    let application = Application::new(ctx)
        .await
        .expect("Failed to build application.");

    let address = format!("http://localhost:{}", application.get_port().unwrap());

    let (_, rx) = tokio::sync::oneshot::channel::<()>();

    // Allow underscore future because it needs to run in background
    // If we `await` it, the tests will hang
    #[allow(clippy::let_underscore_future)]
    let _ = tokio::spawn(async move {
        application
            .start(rx)
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
