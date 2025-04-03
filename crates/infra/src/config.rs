use nittei_utils::create_random_secret;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Config {
    /// Secret code used to create new `Account`s
    pub create_account_secret_code: String,
    /// Port for the application to run on
    pub port: usize,
}

impl Config {
    pub fn new() -> Self {
        let create_account_secret_code = match &nittei_utils::config::APP_CONFIG
            .create_account_secret_code
        {
            Some(code) => code.clone(),
            None => {
                // If we are in debug mode we set a default secret code
                if cfg!(debug_assertions) {
                    let code = "create_account_dev_secret".to_string();
                    info!(
                        "Running in debug mode, using default UNSECURE secret code for creating accounts: {}",
                        code
                    );
                    code
                } else {
                    // Otherwise we generate a random secret code
                    info!(
                        "Did not find CREATE_ACCOUNT_SECRET_CODE environment variable. Going to create one."
                    );
                    let code = create_random_secret(16);
                    info!(
                        "Secret code for creating accounts was generated and set to: {}",
                        code
                    );
                    code
                }
            }
        };
        let port = nittei_utils::config::APP_CONFIG.http_port;

        Self {
            create_account_secret_code,
            port,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
