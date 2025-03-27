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

use std::{net::TcpListener, sync::Arc};

use actix_cors::Cors;
use actix_web::{
    App,
    HttpServer,
    dev::Server,
    middleware::{self},
    web::{self, Data},
};
use futures::lock::Mutex;
use http_logger::NitteiTracingRootSpanBuilder;
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
use tracing::{error, info, warn};
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

/// Configure the Actix server API
/// Add all the routes to the server
pub fn configure_server_api(cfg: &mut web::ServiceConfig) {
    account::configure_routes(cfg);
    calendar::configure_routes(cfg);
    event::configure_routes(cfg);
    schedule::configure_routes(cfg);
    service::configure_routes(cfg);
    status::configure_routes(cfg);
    user::configure_routes(cfg);
}

/// Struct for storing the main application state
pub struct Application {
    /// The Actix server instance
    server: Server,
    /// The port the server is running on
    port: u16,
    /// The application context (database connections, etc.)
    context: NitteiContext,

    /// Shutdown data
    /// Shared state of the server
    shared_state: ServerSharedState,
}

/// Struct for storing the shared state of the server
/// Mainly useful for sharing the shutdown flag between the binary crate and the status endpoint
#[derive(Clone)]
pub struct ServerSharedState {
    /// Flag to indicate if the application is shutting down
    pub is_shutting_down: Arc<Mutex<bool>>,
}

impl Application {
    pub async fn new(context: NitteiContext) -> anyhow::Result<Self> {
        let shared_state = ServerSharedState {
            is_shutting_down: Arc::new(Mutex::new(false)),
        };

        let (server, port) =
            Application::configure_server(context.clone(), shared_state.clone()).await?;

        Application::start_jobs(context.clone());

        Ok(Self {
            server,
            port,
            context,
            shared_state,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Start the background jobs of the application
    /// Note that the jobs are only started if the environment variable NITTEI_REMINDERS_JOB_ENABLED is set to true
    fn start_jobs(context: NitteiContext) {
        if !nittei_utils::config::APP_CONFIG.disable_reminders {
            start_send_reminders_job(context.clone());
            start_reminder_generation_job(context);
        }
    }

    /// Configure the Actix server
    /// This function creates the server and adds all the routes to it
    ///
    /// This adds the following middleware:
    /// - CORS (permissive)
    /// - Compression
    /// - Tracing logger
    async fn configure_server(
        context: NitteiContext,
        shared_state: ServerSharedState,
    ) -> anyhow::Result<(Server, u16)> {
        let port = context.config.port;
        let address = nittei_utils::config::APP_CONFIG.http_host.clone();
        let address_and_port = format!("{}:{}", address, port);
        info!("Starting server on: {}", address_and_port);
        let listener = TcpListener::bind(address_and_port)?;
        let port = listener.local_addr()?.port();

        let server = HttpServer::new(move || {
            let ctx = context.clone();
            let shared_state = shared_state.clone();

            App::new()
                .into_utoipa_app()
                .openapi(ApiDoc::openapi())
                .map(|app| {
                    app.wrap(Cors::permissive())
                        .wrap(middleware::Compress::default())
                        .wrap(TracingLogger::<NitteiTracingRootSpanBuilder>::new())
                        .app_data(Data::new(ctx))
                        .app_data(Data::new(shared_state))
                        .service(web::scope("/api/v1").configure(configure_server_api))
                })
                .openapi_service(|api| {
                    SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api)
                })
                .into_app()
        })
        // Disable signals to avoid conflicts with the signal handler
        // This is handled by the signal handler in the binary and the `schedule_shutdown` function
        .disable_signals()
        // Set the shutdown timeout (time to wait for the server to finish processing requests)
        // Default is 30 seconds
        .shutdown_timeout(nittei_utils::config::APP_CONFIG.server_shutdown_timeout)
        .listen(listener)?
        .workers(4)
        .run();

        Ok((server, port))
    }

    /// Init the default account and start the Actix server
    ///
    /// It also sets up the shutdown handler
    pub async fn start(
        self,
        shutdown_channel: tokio::sync::oneshot::Receiver<()>,
    ) -> anyhow::Result<()> {
        self.setup_shutdown_handler(shutdown_channel);

        self.init_default_account().await?;
        self.server.await.map_err(|e| anyhow::anyhow!(e))
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
    fn setup_shutdown_handler(&self, shutdown_channel: tokio::sync::oneshot::Receiver<()>) {
        let server_handle = self.server.handle();
        let shared_state = self.shared_state.clone();

        // Listen to shutdown channel
        tokio::spawn(async move {
            // Wait for the shutdown channel to receive a message
            if let Err(e) = shutdown_channel.await {
                error!("[server] Failed to listen for shutdown channel: {}", e);
            } else {
                info!("[server] Received shutdown signal",);

                if cfg!(debug_assertions) {
                    // In debug mode, stop the server immediately
                    info!("[server] Stopping server...");
                    server_handle.stop(true).await;
                    info!("[server] Server stopped");
                } else {
                    // In production, do the whole graceful shutdown process

                    // Update flag
                    *shared_state.is_shutting_down.lock().await = true;

                    info!("[server] is_shutting_down flag is now true");

                    let duration = nittei_utils::config::APP_CONFIG.server_shutdown_sleep;

                    info!("[server] Waiting {}s before stopping", duration);

                    // Wait for the timeout
                    tokio::time::sleep(std::time::Duration::from_secs(duration)).await;

                    info!("[server] Stopping server...");

                    // Shutdown the server
                    server_handle.stop(true).await;

                    info!("[server] Server stopped");
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
    ),
)]
struct ApiDoc;
