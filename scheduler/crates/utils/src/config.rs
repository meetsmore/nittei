use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub http_port: u16,
    pub database_url: String,
}

fn parse_config() -> AppConfig {
    let config = Config::builder()
        .add_source(
            config::Environment::with_prefix("APP")
                .try_parsing(true)
                .separator("__"),
        )
        // TODO: move to a file system (default.toml)
        .set_default("http_port", "5000")
        .expect("Failed to set default http_port")
        .set_default(
            "database_url",
            "postgresql://postgres:postgres@localhost:45432/nittei",
        )
        .expect("Failed to set default database_url")
        .build()
        .expect("Failed to build the configuration object");

    let config = config
        .try_deserialize()
        .expect("Failed to deserialize the configuration object");

    config
}

// This is a global variable that will be initialized once
// and will be available throughout the application
// Using global variable is bad practice, but for **immutable** environment variables
// it is acceptable
lazy_static::lazy_static! {
    /// The global configuration object containing all the environment variables
    #[derive(Debug)]
    pub static ref APP_CONFIG: AppConfig = parse_config();
}
