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

use std::net::TcpListener;

use actix_cors::Cors;
use actix_web::{
    dev::Server,
    middleware::{self},
    web::{self, Data},
    App,
    HttpServer,
};
use http_logger::NitteiTracingRootSpanBuilder;
use job_schedulers::{start_reminder_generation_job, start_send_reminders_job};
use nittei_domain::{
    Account,
    AccountIntegration,
    AccountWebhookSettings,
    IntegrationProvider,
    PEMKey,
    ID,
};
use nittei_infra::NitteiContext;
use tracing::{error, info, warn};
use tracing_actix_web::TracingLogger;

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
}

impl Application {
    pub async fn new(context: NitteiContext) -> anyhow::Result<Self> {
        let (server, port) = Application::configure_server(context.clone()).await?;

        Application::start_jobs(context.clone());

        Ok(Self {
            server,
            port,
            context,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Start the background jobs of the application
    /// Note that the jobs are only started if the environment variable NITTEI_REMINDERS_JOB_ENABLED is set to true
    fn start_jobs(context: NitteiContext) {
        if nittei_utils::config::APP_CONFIG.enable_reminders_job {
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
    async fn configure_server(context: NitteiContext) -> anyhow::Result<(Server, u16)> {
        let port = context.config.port;
        let address = nittei_utils::config::APP_CONFIG.http_host.clone();
        let address_and_port = format!("{}:{}", address, port);
        info!("Starting server on: {}", address_and_port);
        let listener = TcpListener::bind(address_and_port)?;
        let port = listener.local_addr()?.port();

        let server = HttpServer::new(move || {
            let ctx = context.clone();

            App::new()
                .wrap(Cors::permissive())
                .wrap(middleware::Compress::default())
                .wrap(TracingLogger::<NitteiTracingRootSpanBuilder>::new())
                .app_data(Data::new(ctx))
                .service(web::scope("/api/v1").configure(configure_server_api))
        })
        .listen(listener)?
        .workers(4)
        .run();

        Ok((server, port))
    }

    /// Init the default account and start the Actix server
    pub async fn start(self) -> anyhow::Result<()> {
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
                warn!("Account not found based on given secret api key - creating default account");
            } else {
                info!("Creating default account with self-generated secret api key");
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
}
