use nittei_utils::create_random_secret;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Config {
    /// Secret code used to create new `Account`s
    pub create_account_secret_code: String,
    /// Maximum allowed duration in millis for querying event instances.
    /// This is used to avoid having clients ask for `CalendarEvents` in a
    /// timespan of several years which will take a lot of time to compute
    /// and is also not very useful information to query about anyways.
    pub event_instances_query_duration_limit: i64,
    /// Maximum allowed duration in millis for querying booking slots
    /// This is used to avoid having clients ask for `BookingSlot`s in a
    /// timespan of several years which will take a lot of time to compute
    /// and is also not very useful information to query about anyways.
    pub booking_slots_query_duration_limit: i64,
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

        const DAYS_62: i64 = 1000 * 60 * 60 * 24 * 62;
        const DAYS_101: i64 = 1000 * 60 * 60 * 24 * 101;

        Self {
            create_account_secret_code,
            event_instances_query_duration_limit: DAYS_62,
            booking_slots_query_duration_limit: DAYS_101,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
