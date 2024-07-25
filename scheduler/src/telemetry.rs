use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber() {
    // Filter the spans that are shown based on the RUST_LOG env var or the default value ("info")
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // TODO: add the `env` on all logs

    // If the binary is compiled in debug mode (aka for development)
    // use the compact format for logs
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    } else {
        // In production, use the JSON format for logs
        let service_name =
            std::env::var("SERVICE_NAME").unwrap_or_else(|_| "unknown service".to_string());
        let service_version =
            std::env::var("SERVICE_VERSION").unwrap_or_else(|_| "unknown version".to_string());
        let service_env =
            std::env::var("SERVICE_ENV").unwrap_or_else(|_| "unknown env".to_string());

        // Set the global propagator to trace context propagator
        global::set_text_map_propagator(TraceContextPropagator::new());

        // First, create a OTLP exporter builder. Configure it as you need.
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .http()
            .with_protocol(opentelemetry_otlp::Protocol::HttpJson);

        // Then pass it into pipeline builder
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
                opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name.clone()),
                    opentelemetry::KeyValue::new("service.version", service_version),
                    opentelemetry::KeyValue::new("deployment.environment", service_env),
                ]),
            ))
            .with_exporter(otlp_exporter)
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .unwrap();

        let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        // Create a `tracing` layer to emit spans as structured logs to stdout
        let formatting_layer = BunyanFormattingLayer::new(service_name, std::io::stdout);

        // Combine layers into a single subscriber
        let subscriber = Registry::default()
            .with(env_filter)
            .with(telemetry_layer)
            .with(JsonStorageLayer)
            .with(formatting_layer);

        // Set the global subscriber
        tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set global subscriber");
    }
}
