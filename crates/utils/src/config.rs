use std::sync::LazyLock;

use config::Config;
use serde::Deserialize;

/// Application configuration (main)
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    //// The host to bind the HTTP server to
    //// Default is 127.0.0.1
    //// Env var: NITTEI__HTTP_HOST
    pub http_host: String,

    /// The port to bind the HTTP server to
    /// Default is 5000
    /// Env var: NITTEI__HTTP_PORT
    pub http_port: usize,

    /// The sleep time for the HTTP server shutdown (in seconds)
    /// Default is 5 seconds
    /// Env var: NITTEI__SERVER_SHUTDOWN_SLEEP
    pub server_shutdown_sleep: u64,

    /// The shutdown timeout for the HTTP server (in seconds)
    /// Default is 10 seconds
    /// Env var: NITTEI__SERVER_SHUTDOWN_TIMEOUT
    pub server_shutdown_timeout: u64,

    /// Pg config
    pub pg: PgConfig,

    /// The secret code to create accounts (superadmin)
    /// Default is a random 16 characters string
    /// Env var: NITTEI__CREATE_ACCOUNT_SECRET_CODE
    pub create_account_secret_code: Option<String>,

    /// This is a flag for disabling the reminders features
    /// Be careful, as this impacts what is saved in database
    /// So changing from one to the other is only safe if the features aren't used
    /// Otherwise, data might be missing
    ///
    /// Default is false
    /// Env var: NITTEI__DISABLE_REMINDERS
    pub disable_reminders: bool,

    /// Max number of events returned that can be returned at once by search (u16)
    /// Default to 5000
    /// Env var: NITTEI__MAX_EVENTS_RETURNED_BY_SEARCH
    pub max_events_returned_by_search: u16,

    /// Maximum allowed duration in millis for querying event instances.
    /// This is used to avoid having clients ask for `CalendarEvents` in a
    /// timespan of several years which will take a lot of time to compute
    /// and is also not very useful information to query about anyways.
    /// Default is 200 days
    /// Env var: NITTEI__EVENT_INSTANCES_QUERY_DURATION_LIMIT
    pub event_instances_query_duration_limit: i64,

    /// Maximum allowed duration in millis for querying booking slots
    /// This is used to avoid having clients ask for `BookingSlot`s in a
    /// timespan of several years which will take a lot of time to compute
    /// and is also not very useful information to query about anyways.
    /// Default is 100 days
    /// Env var: NITTEI__BOOKING_SLOTS_QUERY_DURATION_LIMIT
    pub booking_slots_query_duration_limit: i64,

    /// The account configuration
    /// This is used to find the superadmin account
    pub account: Option<AccountConfig>,

    /// The observability configuration
    /// This is used to configure the observability tools
    pub observability: Option<ObservabilityConfig>,
}

/// Observability configuration
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ObservabilityConfig {
    /// Service name for the tracing
    /// Default is "unknown service"
    /// Env var: NITTEI__OBSERVABILITY__SERVICE_NAME
    pub service_name: Option<String>,

    /// Service version for the tracing
    /// Default is "unknown version"
    /// Env var: NITTEI__OBSERVABILITY__SERVICE_VERSION
    pub service_version: Option<String>,

    /// Service environment for the tracing
    /// Default is "unknown env"
    /// Env var: NITTEI__OBSERVABILITY__SERVICE_ENV
    pub service_env: Option<String>,

    /// The tracing sample rate
    /// Default is 0.1
    /// Env var: NITTEI__OBSERVABILITY__TRACING_SAMPLE_RATE
    pub tracing_sample_rate: Option<f64>,

    /// The OTLP tracing endpoint
    /// Env var: NITTEI__OBSERVABILITY__OTLP_TRACING_ENDPOINT
    pub otlp_tracing_endpoint: Option<String>,

    /// The Datadog tracing endpoint
    /// Env var: NITTEI__OBSERVABILITY__DATADOG_TRACING_ENDPOINT
    pub datadog_tracing_endpoint: Option<String>,
}

/// Account configuration
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AccountConfig {
    /// Secret key to find the superadmin account
    /// Env var: NITTEI__ACCOUNT__SECRET_KEY
    pub secret_key: Option<String>,

    /// The account ID
    /// Used only if the account is not found by the secret key
    /// Env var: NITTEI__ACCOUNT__ID
    pub id: Option<String>,

    /// The account name
    /// Used only if the account is not found by the secret key
    /// Env var: NITTEI__ACCOUNT__WEBHOOK_URL
    pub webhook_url: Option<String>,

    /// Pub key
    /// Used only if the account is not found by the secret key
    /// Env var: NITTEI__ACCOUNT__PUB_KEY
    pub pub_key: Option<String>,

    /// Google integration configuration
    /// Used only if the account is not found by the secret key
    pub google: Option<IntegrationConfig>,

    /// Outlook integration configuration
    /// Used only if the account is not found by the secret key
    pub outlook: Option<IntegrationConfig>,
}

/// Integration configuration
/// This is used for Google and Outlook integrations
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct IntegrationConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

/// Postgres configuration
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct PgConfig {
    /// The database URL
    /// Default is postgresql://postgres:postgres@localhost:45432/nittei
    /// Env var: NITTEI__PG__DATABASE_URL
    pub database_url: String,

    /// This is a flag to skip the database migration
    /// Default is false
    /// Env var: NITTEI__PG__SKIP_MIGRATIONS
    pub skip_migrations: bool,

    /// The minimum number of connections to the database
    /// Default is 2
    /// Env var: NITTEI__PG__MIN_CONNECTIONS
    pub min_connections: u32,

    /// The maximum number of connections to the database
    /// Default is 5
    /// Env var: NITTEI__PG__MAX_CONNECTIONS
    pub max_connections: u32,
}

/// Parse the configuration from the environment variables
/// and return the configuration object
///
/// This function will panic if the configuration is not valid !
///
/// This called by the `APP_CONFIG` global variable (lazy_static)
fn parse_config() -> AppConfig {
    #[allow(clippy::expect_used)]
    let config = Config::builder()
        .add_source(
            config::Environment::with_prefix("NITTEI")
                .try_parsing(true)
                .separator("__"),
        )
        .set_default("http_host", "127.0.0.1")
        .expect("Failed to set default host")
        .set_default("http_port", 5000)
        .expect("Failed to set default port")
        .set_default("server_shutdown_sleep", 5)
        .expect("Failed to set default server_shutdown_sleep")
        .set_default("server_shutdown_timeout", 10)
        .expect("Failed to set default server_shutdown_timeout")
        .set_default("pg.skip_migrations", false)
        .expect("Failed to set default pg.skip_migrations")
        .set_default("pg.min_connections", 2)
        .expect("Failed to set default pg.min_connections")
        .set_default("pg.max_connections", 2)
        .expect("Failed to set default pg.max_connections")
        .set_default("disable_reminders", false)
        .expect("Failed to set default disable_reminders")
        .set_default("max_events_returned_by_search", 5000)
        .expect("Failed to set default max_events_returned_by_search")
        .set_default(
            "event_instances_query_duration_limit",
            200_i64 * 24 * 60 * 60 * 1000, // 200 days
        )
        .expect("Failed to set default event_instances_query_duration_limit")
        .set_default(
            "booking_slots_query_duration_limit",
            100_i64 * 24 * 60 * 60 * 1000, // 100 days
        )
        .expect("Failed to set default booking_slots_query_duration_limit")
        .set_default(
            "pg.database_url",
            "postgresql://postgres:postgres@localhost:45432/nittei",
        )
        .expect("Failed to set default pg.database_url")
        .build()
        .expect("Failed to build the configuration object");

    #[allow(clippy::expect_used)]
    let config = config
        .try_deserialize()
        .expect("Failed to deserialize the configuration object");

    config
}

// This is a global variable that will be initialized once
// and will be available throughout the application
// Using global variable is bad practice, but for **immutable** environment variables
// it is acceptable
pub static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(parse_config);
