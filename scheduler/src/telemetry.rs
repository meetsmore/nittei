use std::{fmt, io};

use chrono::{DateTime, Utc};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{Config, TracerProvider};
use opentelemetry_sdk::Resource;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use tracing_subscriber::{fmt::time::FormatTime, EnvFilter};

/// Struct used to keep the format for having custom formatting of timestamps in the logs
struct TracingChronoTimer;

// Implement the `FormatTime` trait required by tracing_subscriber
// for the `TracingChronoTimer` struct
impl FormatTime for TracingChronoTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> fmt::Result {
        let now: DateTime<Utc> = Utc::now();
        write!(w, "{}", now.to_rfc3339())
    }
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber() {
    // Filter the spans that are shown based on the RUST_LOG env var or the default value ("info")
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    // TODO: add the `env` on all logs

    // If the binary is compiled in debug mode, use the compact format for logs
    // In other words, if we are in dev
    // if cfg!(debug_assertions) {
    //     tracing_subscriber::fmt()
    //         .compact()
    //         .with_env_filter(env_filter)
    //         .init();
    // } else {
    global::set_text_map_propagator(TraceContextPropagator::new());
    // let provider = TracerProvider::builder()
    //     .with_batch_exporter(opentelemetry_sdk::runtime::Tokio)
    //     .build();
    // First, create a OTLP exporter builder. Configure it as you need.
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpJson);
    // .with_endpoint("http://localhost:4318/v1/traces");
    // Then pass it into pipeline builder
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
            opentelemetry_sdk::Resource::new(vec![
                opentelemetry::KeyValue::new("service.name", "scheduler-service"),
                opentelemetry::KeyValue::new("service.version", "0.1.0"),
                opentelemetry::KeyValue::new("deployment.environment", "development"),
            ]),
        ))
        .with_exporter(otlp_exporter)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();
    // let tracer = global::tracer("scheduler-service");

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create a `tracing` layer to emit spans as structured logs to stdout
    let formatting_layer = BunyanFormattingLayer::new("scheduler-service".into(), std::io::stdout);
    // Otherwise, use the JSON format for logs
    // let json_logging_layer = tracing_subscriber::fmt::layer()
    //     .json()
    //     .with_timer(TracingChronoTimer {})
    //     .with_current_span(false)
    //     .with_writer(io::stdout);

    // Combine layers into a single subscriber
    let subscriber = Registry::default()
        .with(env_filter)
        .with(telemetry_layer)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    // .with(json_logging_layer);

    // Set the global subscriber
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
    // }
}
