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
use http_logger::metadata_middleware;
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
use utoipa::{
    Modify,
    OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    http_logger::{NitteiTracingOnFailure, NitteiTracingOnResponse, NitteiTracingSpanBuilder},
    shared::auth::NITTEI_X_API_KEY_HEADER,
};

/// Configure the Actix server API
/// Add all the routes to the server
pub fn configure_server_api() -> OpenApiRouter {
    OpenApiRouter::new()
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

    /// Listener for the server
    listener: TcpListener,

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

        let (server, listener) =
            Application::configure_server(context.clone(), shared_state.clone()).await?;

        Application::start_jobs(context.clone());

        Ok(Self {
            server,
            listener,
            context,
            shared_state,
        })
    }

    pub fn get_port(&self) -> anyhow::Result<u16> {
        Ok(self.listener.local_addr()?.port())
    }

    /// Start the background jobs of the application
    /// Note that the jobs are only started if the environment variable NITTEI_REMINDERS_JOB_ENABLED is set to true
    fn start_jobs(context: NitteiContext) {
        if !nittei_utils::config::APP_CONFIG.disable_reminders {
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
    /// - Sensitive headers
    /// - Compression
    /// - Catch panics
    async fn configure_server(
        context: NitteiContext,
        shared_state: Arc<Mutex<ServerSharedState>>,
    ) -> anyhow::Result<(Router, TcpListener)> {
        let api_router = configure_server_api();

        let sensitive_headers = vec![
            header::AUTHORIZATION,
            header::HeaderName::from_static(NITTEI_X_API_KEY_HEADER),
        ];

        let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .nest("/api/v1", api_router)
            .layer(axum::middleware::from_fn(metadata_middleware))
            .layer(
                ServiceBuilder::new()
                    // Mark the `Authorization` header as sensitive so it doesn't show in logs
                    .layer(SetSensitiveHeadersLayer::new(sensitive_headers))
                    .layer(CorsLayer::permissive())
                    .layer(RequestDecompressionLayer::new())
                    .layer(CompressionLayer::new())
                    // Catch panics and convert them into responses.
                    .layer(CatchPanicLayer::new())
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(NitteiTracingSpanBuilder {})
                            .on_request(())
                            .on_response(NitteiTracingOnResponse {})
                            .on_failure(NitteiTracingOnFailure {}),
                    ),
            )
            .layer(Extension(context.clone()))
            .layer(Extension(shared_state.clone()))
            .split_for_parts();

        let router =
            router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()));

        let port = context.config.port;
        let address = nittei_utils::config::APP_CONFIG.http_host.clone();
        let address_and_port = format!("{address}:{port}");

        let listener = TcpListener::bind(address_and_port).await?;
        info!("[server] Will start server on: {}", listener.local_addr()?);

        Ok((router, listener))
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

        // Create a shutdown signal channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shared_state.lock().await.shutdown_tx = Some(shutdown_tx);

        info!("[server] Server started");
        axum::serve(self.listener, self.server)
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
                info!("[init_default_account] Using provided secret api key");
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
                info!(
                    "[init_default_account] Creating default account with self-generated secret api key"
                );
            } else {
                warn!(
                    "[init_default_account] Account not found based on given secret api key - creating default account"
                );
            }

            let mut account = Account::default();
            let account_id = nittei_utils::config::APP_CONFIG
                .account
                .as_ref()
                .and_then(|a| a.id.clone())
                .unwrap_or_default()
                .parse::<ID>()
                .unwrap_or_default();

            info!("[init_default_account] Using account id: {}", account_id);
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
                    Err(e) => warn!(
                        "[init_default_account] Invalid ACCOUNT_PUB_KEY provided: {:?}",
                        e
                    ),
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
                            "{account_google_client_secret_env} should be specified also when {account_google_client_id_env} is specified."
                        )
                    });
                let google_redirect_uri = google_config
                    .map(|g| g.redirect_uri.clone())
                    .unwrap_or_else(|| {
                        panic!(
                            "{account_google_redirect_uri_env} should be specified also when {account_google_client_id_env} is specified."
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
                            "{account_outlook_client_secret_env} should be specified also when {account_outlook_client_id_env} is specified."
                        )
                    });
                let outlook_redirect_uri = outlook_config
                    .map(|o| o.redirect_uri.clone())
                    .unwrap_or_else(|| {
                        panic!(
                            "{account_outlook_redirect_uri_env} should be specified also when {account_outlook_client_id_env} is specified."
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
                    info!("[init_default_account] Account created: {:?}", account.id);
                } else {
                    error!(
                        "[init_default_account] Account not created {:?}",
                        account.id
                    );
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

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Nittei API",
        version = "1.0.0",
        description = "OpenAPI documentation for the Nittei API",
    ),
    tags(
        {name = "Account", description = "Account API endpoints"}, 
        {name = "Calendar", description = "Calendar API endpoints"},
        {name = "Event", description = "Event API endpoints"},
        {name = "Schedule", description = "Schedule API endpoints"},
        {name = "Service", description = "Service API endpoints"},
        {name = "Status", description = "Status API endpoints"},
        {name = "User", description = "User API endpoints"},
    ),
    modifiers(&SecurityAddon),
    paths(
        // Account
        account::account_search_events::account_search_events_controller,
        account::create_account::create_account_controller,
        account::get_account::get_account_controller,
        account::set_account_pub_key::set_account_pub_key_controller,
        account::set_account_webhook::set_account_webhook_controller,
        account::delete_account_webhook::delete_account_webhook_controller,
        account::add_account_integration::add_account_integration_controller,
        account::remove_account_integration::remove_account_integration_controller,

        // Calendar
        calendar::get_calendars::get_calendars_controller,
        calendar::get_calendars::get_calendars_admin_controller,
        calendar::get_calendars_by_meta::get_calendars_by_meta_controller,
        calendar::get_calendar::get_calendar_controller,
        calendar::get_calendar::get_calendar_admin_controller,
        calendar::delete_calendar::delete_calendar_controller,
        calendar::delete_calendar::delete_calendar_admin_controller,
        calendar::update_calendar::update_calendar_controller,
        calendar::update_calendar::update_calendar_admin_controller,
        calendar::get_calendar_events::get_calendar_events_controller,
        calendar::get_calendar_events::get_calendar_events_admin_controller,
        calendar::get_google_calendars::get_google_calendars_controller,
        calendar::get_google_calendars::get_google_calendars_admin_controller,
        calendar::get_outlook_calendars::get_outlook_calendars_controller,
        calendar::get_outlook_calendars::get_outlook_calendars_admin_controller,
        calendar::remove_sync_calendar::remove_sync_calendar_admin_controller,
        calendar::add_sync_calendar::add_sync_calendar_admin_controller,

        // Event
        event::create_event::create_event_controller,
        event::create_event::create_event_admin_controller,
        event::delete_event::delete_event_controller,
        event::delete_event::delete_event_admin_controller,
        event::delete_many_events::delete_many_events_admin_controller,
        event::get_event::get_event_controller,
        event::get_event::get_event_admin_controller,
        event::get_event_by_external_id::get_event_by_external_id_admin_controller,
        event::get_event_instances::get_event_instances_controller,
        event::get_event_instances::get_event_instances_admin_controller,
        event::get_events_by_calendars::get_events_by_calendars_controller,
        event::get_events_by_meta::get_events_by_meta_controller,
        event::search_events::search_events_controller,
        event::update_event::update_event_controller,
        event::update_event::update_event_admin_controller,

        // User
        user::create_user::create_user_controller,
        user::get_me::get_me_controller,
        user::get_user::get_user_controller,
        user::get_user_by_external_id::get_user_by_external_id_controller,
        user::get_multiple_users_freebusy::get_multiple_freebusy_controller,
        user::get_user_freebusy::get_freebusy_controller,
        user::update_user::update_user_controller,
        user::delete_user::delete_user_controller,
        user::oauth_integration::oauth_integration_controller,
        user::remove_integration::remove_integration_controller,
        user::oauth_integration::oauth_integration_admin_controller,
        user::remove_integration::remove_integration_admin_controller,
    ),
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
            )
        }
    }
}
