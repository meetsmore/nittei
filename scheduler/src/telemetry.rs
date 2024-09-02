use opentelemetry::global::{self, set_error_handler};
use opentelemetry_datadog::{new_pipeline, ApiVersion};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{self, RandomIdGenerator, Sampler, Tracer},
};
use tracing::warn;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber() {
    // Filter the spans that are shown based on the RUST_LOG env var or the default value ("info")
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

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

        // Get the tracer - if no endpoint is provided, tracing will be disabled
        let tracer = get_tracer(service_name, service_version, service_env);

        // Create a telemetry layer if a tracer is available
        let telemetry_layer =
            tracer.map(|tracer| tracing_opentelemetry::layer().with_tracer(tracer));

        // Combine layers into a single subscriber
        if telemetry_layer.is_some() {
            let subscriber = Registry::default()
                .with(env_filter)
                .with(telemetry_layer.unwrap())
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_current_span(false),
                );

            // Set the global subscriber
            tracing::subscriber::set_global_default(subscriber)
                .expect("Unable to set global subscriber");
        } else {
            // If no tracer is available, do not include telemetry layer
            let subscriber = Registry::default().with(env_filter).with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_current_span(false),
            );

            // Set the global subscriber
            tracing::subscriber::set_global_default(subscriber)
                .expect("Unable to set global subscriber");
        }

        // Set a global error handler to log the tracing internal errors to the console
        set_error_handler(|e| {
            warn!("Error when exporting traces: {}", e);
        })
        .expect("Failed to set global error handler");
    }
}

/// Get the tracer
fn get_tracer(
    service_name: String,
    service_version: String,
    service_env: String,
) -> Option<Tracer> {
    let otlp_endpoint = std::env::var("OTLP_TRACING_ENDPOINT");
    let datadog_endpoint = std::env::var("DATADOG_TRACING_ENDPOINT");

    if let Ok(datadog_endpoint) = datadog_endpoint {
        Some(get_tracer_datadog(
            datadog_endpoint,
            service_name,
            service_version,
            service_env,
        ))
    } else if let Ok(otlp_endpoint) = otlp_endpoint {
        Some(get_tracer_otlp(
            otlp_endpoint,
            service_name,
            service_version,
            service_env,
        ))
    } else {
        warn!("No tracing endpoints provided (DATADOG_TRACING_ENDPOINT or OTLP_TRACING_ENDPOINT), tracing will be disabled");
        None
    }
}

/// Get the tracer based on the tracing endpoint
fn get_tracer_datadog(
    datadog_endpoint: String,
    service_name: String,
    service_version: String,
    service_env: String,
) -> Tracer {
    new_pipeline()
        .with_service_name(service_name)
        .with_version(service_version)
        .with_env(service_env)
        .with_api_version(ApiVersion::Version05)
        .with_agent_endpoint(datadog_endpoint)
        .with_trace_config(
            trace::Config::default()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap()
}

fn get_tracer_otlp(
    otlp_endpoint: String,
    service_name: String,
    service_version: String,
    service_env: String,
) -> Tracer {
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpJson)
        .with_endpoint(otlp_endpoint);

    // Then pass it into pipeline builder
    opentelemetry_otlp::new_pipeline()
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
        .unwrap()
}
