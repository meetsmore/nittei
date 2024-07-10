use std::fmt;

use chrono::{DateTime, Utc};
use tracing_subscriber::{fmt::time::FormatTime, EnvFilter};

/// Struct used to keep the format for having custom formatting of timestamps in the logs
struct TracingChronoTimer {
    format: String,
}

impl TracingChronoTimer {
    fn with_format(format: String) -> Self {
        Self { format }
    }
}

// Implement the `FormatTime` trait required by tracing_subscriber
// for the `TracingChronoTimer` struct
impl FormatTime for TracingChronoTimer {
    fn format_time(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        let now: DateTime<Utc> = Utc::now();
        write!(w, "{}", now.format(&self.format))
    }
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber() {
    // Filter the spans that are shown based on the RUST_LOG env var or the default value ("info")
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // TODO: add the `env` on all logs

    // If the binary is compiled in debug mode, use the compact format for logs
    // In other words, if we are in dev
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    } else {
        // Otherwise, use the JSON format for logs
        tracing_subscriber::fmt()
            .json()
            .with_timer(TracingChronoTimer::with_format(
                "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
            ))
            .with_env_filter(env_filter)
            .with_current_span(false)
            .init();
    }
}
