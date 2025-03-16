mod account;
mod calendar;
mod error;
mod event;
mod http_logger;
mod job_schedulers;
mod schedule;
mod service;
mod shared;
mod status;
mod user;

use std::sync::Arc;

use axum::{Extension, Router, http::header};
use futures::lock::Mutex;
use job_schedulers::{start_reminder_generation_job, start_send_reminders_job};
use nittei_domain::{
    Account,
    AccountIntegration,
    AccountWebhookSettings,
    ID,
    IntegrationProvider,
    PEMKey,
};
use nittei_infra::NitteiContext;
use tokio::{
    net::TcpListener,
    sync::oneshot::{self, Sender},
};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    decompression::RequestDecompressionLayer,
    sensitive_headers::SetSensitiveHeadersLayer,
    trace::TraceLayer,
};
use tracing::{error, info, warn};

/// Configure the Actix server API
/// Add all the routes to the server
pub fn configure_server_api() -> Router {
    Router::new()
        .merge(account::configure_routes())
        .merge(calendar::configure_routes())
        .merge(event::configure_routes())
        .merge(schedule::configure_routes())
        .merge(service::configure_routes())
        .merge(status::configure_routes())
        .merge(user::configure_routes())
}

/// Struct for storing the main application state
pub struct Application {
    /// The Axum server instance
    server: Router,
    /// The port the server is running on
    // port: u16,
    /// The application context (database connections, etc.)
    context: NitteiContext,

    /// Shutdown data
    /// Shared state of the server
    shared_state: Arc<Mutex<ServerSharedState>>,
}
/// Struct for storing the shared state of the server
/// Mainly useful for sharing the shutdown flag between the binary crate and the status endpoint
pub struct ServerSharedState {
    /// Channel to send shutdown signal
    pub shutdown_tx: Option<Sender<()>>,
}

impl Application {
    pub async fn new(context: NitteiContext) -> anyhow::Result<Self> {
        let shared_state = Arc::new(Mutex::new(ServerSharedState { shutdown_tx: None }));

        let server = Application::configure_server(context.clone(), shared_state.clone()).await?;

        Application::start_jobs(context.clone());

        Ok(Self {
            server,
            context,
            shared_state,
        })
    }

    /// Start the background jobs of the application
    /// Note that the jobs are only started if the environment variable NITTEI_REMINDERS_JOB_ENABLED is set to true
    fn start_jobs(context: NitteiContext) {
        if nittei_utils::config::APP_CONFIG.enable_reminders_job {
            start_send_reminders_job(context.clone());
            start_reminder_generation_job(context);
        }
    }

    /// Configure the Axum server
    /// This function creates the server and adds all the routes to it
    ///
    /// This adds the following middleware:
    /// - CORS (permissive)
    /// - Compression
    /// - Tracing logger
    async fn configure_server(
        context: NitteiContext,
        shared_state: Arc<Mutex<ServerSharedState>>,
    ) -> anyhow::Result<Router> {
        let api_router = configure_server_api();

        let sensitive_headers = vec![header::AUTHORIZATION];

        let server = Router::new()
            .nest("/api/v1", api_router)
            .layer(
                ServiceBuilder::new()
                    // Mark the `Authorization` header as sensitive so it doesn't show in logs
                    .layer(SetSensitiveHeadersLayer::new(sensitive_headers))
                    .layer(CorsLayer::permissive())
                    .layer(RequestDecompressionLayer::new())
                    .layer(CompressionLayer::new())
                    // Catch panics and convert them into responses.
                    .layer(CatchPanicLayer::new())
                    .layer(TraceLayer::new_for_http()),
            )
            .layer(Extension(context.clone()))
            .layer(Extension(shared_state.clone()));

        Ok(server)
    }

    /// Init the default account and start the Actix server
    ///
    /// It also sets up the shutdown handler
    pub async fn start(
        mut self,
        shutdown_channel: tokio::sync::oneshot::Receiver<()>,
    ) -> anyhow::Result<()> {
        self.setup_shutdown_handler(shutdown_channel);

        self.init_default_account().await?;

        let port = nittei_utils::config::APP_CONFIG.http_port;
        let address = nittei_utils::config::APP_CONFIG.http_host.clone();
        let address_and_port = format!("{}:{}", address, port);

        // Create a shutdown signal channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shared_state.lock().await.shutdown_tx = Some(shutdown_tx);

        info!("Starting server on: {}", address_and_port);
        let listener = TcpListener::bind(address_and_port).await?;

        axum::serve(listener, self.server)
            .with_graceful_shutdown(async {
                shutdown_rx.await.unwrap_or_else(|e| {
                    error!("[server] Failed to listen for shutdown signal: {}", e)
                });
                info!("[server] Server stopped");
            })
            .await
            .unwrap_or_else(|e| {
                error!("[server] Server error: {:?}", e);
                // Exit the process with an error code
                std::process::exit(1);
            });

        info!("[server] Server closed");

        Ok(())
    }

    /// Initialize the default account
    /// The default account is created if it doesn't exist
    async fn init_default_account(&self) -> anyhow::Result<()> {
        let secret_api_key_option = nittei_utils::config::APP_CONFIG
            .account
            .as_ref()
            .and_then(|a| a.secret_key.clone());

        let secret_api_key = match &secret_api_key_option {
            Some(key) => {
                info!("Using provided secret api key");
                key.to_owned()
            }
            None => Account::generate_secret_api_key(),
        };

        if self
            .context
            .repos
            .accounts
            .find_by_apikey(&secret_api_key)
            .await?
            .is_none()
        {
            if secret_api_key_option.is_none() {
                info!("Creating default account with self-generated secret api key");
            } else {
                warn!("Account not found based on given secret api key - creating default account");
            }

            let mut account = Account::default();
            let account_id = nittei_utils::config::APP_CONFIG
                .account
                .as_ref()
                .and_then(|a| a.id.clone())
                .unwrap_or_default()
                .parse::<ID>()
                .unwrap_or_default();

            info!("Using account id: {}", account_id);
            account.id = account_id;
            account.secret_api_key = secret_api_key.clone();
            account.settings.webhook = nittei_utils::config::APP_CONFIG
                .account
                .as_ref()
                .and_then(|a| a.webhook_url.clone())
                .map(|url| AccountWebhookSettings {
                    url,
                    key: Default::default(),
                });

            if let Some(mut verification_key) = nittei_utils::config::APP_CONFIG
                .account
                .as_ref()
                .and_then(|a| a.pub_key.clone())
            {
                verification_key = verification_key.replace("\\n", "\n");
                match PEMKey::new(verification_key) {
                    Ok(k) => account.set_public_jwt_key(Some(k)),
                    Err(e) => warn!("Invalid ACCOUNT_PUB_KEY provided: {:?}", e),
                };
            }

            self.context.repos.accounts.insert(&account).await?;

            let account_google_client_id_env = "NITTEI__ACCOUNT__GOOGLE__CLIENT_ID";
            let account_google_client_secret_env = "NITTEI__ACCOUNT__GOOGLE__CLIENT_SECRET";
            let account_google_redirect_uri_env = "NITTEI__ACCOUNT__GOOGLE__REDIRECT_URI";
            let google_config = nittei_utils::config::APP_CONFIG
                .account
                .as_ref()
                .and_then(|a| a.google.as_ref());
            if let Some(google_client_id) = google_config.map(|g| g.client_id.clone()) {
                let google_client_secret = google_config
                    .map(|g| g.client_secret.clone())
                    .unwrap_or_else(|| {
                        panic!(
                            "{} should be specified also when {} is specified.",
                            account_google_client_secret_env, account_google_client_id_env
                        )
                    });
                let google_redirect_uri = google_config
                    .map(|g| g.redirect_uri.clone())
                    .unwrap_or_else(|| {
                        panic!(
                            "{} should be specified also when {} is specified.",
                            account_google_redirect_uri_env, account_google_client_id_env
                        )
                    });
                self.context
                    .repos
                    .account_integrations
                    .insert(&AccountIntegration {
                        account_id: account.id.clone(),
                        client_id: google_client_id,
                        client_secret: google_client_secret,
                        redirect_uri: google_redirect_uri,
                        provider: IntegrationProvider::Google,
                    })
                    .await?;
            }

            let account_outlook_client_id_env = "NITTEI__ACCOUNT__OUTLOOK__CLIENT_ID";
            let account_outlook_client_secret_env = "NITTEI__ACCOUNT__OUTLOOK__CLIENT_SECRET";
            let account_outlook_redirect_uri_env = "NITTEI__ACCOUNT__OUTLOOK__REDIRECT_URI";
            let outlook_config = nittei_utils::config::APP_CONFIG
                .account
                .as_ref()
                .and_then(|a| a.outlook.as_ref());
            if let Some(outlook_client_id) = outlook_config.map(|o| o.client_id.clone()) {
                let outlook_client_secret = outlook_config
                    .map(|o| o.client_secret.clone())
                    .unwrap_or_else(|| {
                        panic!(
                            "{} should be specified also when {} is specified.",
                            account_outlook_client_secret_env, account_outlook_client_id_env
                        )
                    });
                let outlook_redirect_uri = outlook_config
                    .map(|o| o.redirect_uri.clone())
                    .unwrap_or_else(|| {
                        panic!(
                            "{} should be specified also when {} is specified.",
                            account_outlook_redirect_uri_env, account_outlook_client_id_env
                        )
                    });
                self.context
                    .repos
                    .account_integrations
                    .insert(&AccountIntegration {
                        account_id: account.id.clone(),
                        client_id: outlook_client_id,
                        client_secret: outlook_client_secret,
                        redirect_uri: outlook_redirect_uri,
                        provider: IntegrationProvider::Outlook,
                    })
                    .await?;

                // Check account is created
                if let Some(account) = self
                    .context
                    .repos
                    .accounts
                    .find_by_apikey(&secret_api_key)
                    .await?
                {
                    info!("Account created: {:?}", account.id);
                } else {
                    error!("Account not created {:?}", account.id);
                }
            }
        };
        Ok(())
    }

    /// Setup the shutdown handler
    fn setup_shutdown_handler(&mut self, shutdown_channel: tokio::sync::oneshot::Receiver<()>) {
        // Clone shutdown_tx before entering the async block
        let shared_state = self.shared_state.clone();

        // Listen to shutdown channel
        tokio::spawn(async move {
            // Wait for the shutdown channel to receive a message
            if let Err(e) = shutdown_channel.await {
                error!(
                    "[shutdown_handler] Failed to listen for shutdown channel: {}",
                    e
                );
            } else {
                info!("[shutdown_handler] Received shutdown signal",);

                if cfg!(debug_assertions) {
                    // In debug mode, stop the server immediately
                    info!("[shutdown_handler] Stopping server...");
                    if let Some(server_handle) = shared_state.lock().await.shutdown_tx.take() {
                        server_handle.send(()).unwrap_or_else(|_| {
                            error!("[shutdown_handler] Failed to send shutdown signal");
                            // Exit the process with an error code
                            std::process::exit(1);
                        });
                    }
                    info!("[shutdown_handler] api crate - shutdown complete");
                } else {
                    // In production, do the whole graceful shutdown process (wait for a timeout before stopping the server)

                    let duration = nittei_utils::config::APP_CONFIG.server_shutdown_sleep;

                    info!("[shutdown_handler] Waiting {}s before stopping", duration);

                    // Wait for the timeout
                    tokio::time::sleep(std::time::Duration::from_secs(duration)).await;

                    info!("[shutdown_handler] Stopping server...");

                    // Shutdown the server
                    if let Some(server_handle) = shared_state.lock().await.shutdown_tx.take() {
                        server_handle.send(()).unwrap_or_else(|_| {
                            error!("[shutdown_handler] Failed to send shutdown signal");
                            // Exit the process with an error code
                            std::process::exit(1);
                        });
                    }

                    info!("[shutdown_handler] api crate - shutdown complete");
                }
            }
        });
    }
}
