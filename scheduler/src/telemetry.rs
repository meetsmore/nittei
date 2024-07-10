use std::fmt;

use chrono::{DateTime, Utc};
use tracing_subscriber::{fmt::time::FormatTime, EnvFilter};

/// Custom formatting for chrono timer
struct CustomChronoTimer {
    format: String,
}

impl CustomChronoTimer {
    fn with_format(format: String) -> Self {
        Self { format }
    }
}

impl FormatTime for CustomChronoTimer {
    fn format_time(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        let now: DateTime<Utc> = Utc::now();
        write!(w, "{}", now.format(&self.format))
    }
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber() {
    // Filter the spans that are shown based on the RUST_LOG env var or the default value.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .with_timer(CustomChronoTimer::with_format(
                "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
            ))
            .with_env_filter(env_filter)
            .with_current_span(false)
            .init();
    }
}
