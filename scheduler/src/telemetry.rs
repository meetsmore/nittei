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
                .with(tracing_subscriber::fmt::layer().json())
                .with(telemetry_layer.unwrap());

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
/// This is for the (unofficial) Datadog exporter
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
                .with_sampler(get_sampler())
                .with_id_generator(RandomIdGenerator::default()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap()
}

/// Get the tracer based on the OTLP endpoint
/// This is for the OpenTelemetry Protocol (OTLP) exporter
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
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name.clone()),
                    opentelemetry::KeyValue::new("service.version", service_version),
                    opentelemetry::KeyValue::new("deployment.environment", service_env),
                ]))
                .with_sampler(get_sampler()),
        )
        .with_exporter(otlp_exporter)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap()
}

/// Get the sampler to be used
/// This is a parent-based sampler, so if a parent exists, always sample
/// If there is no parent, then this is based on the TRACING_SAMPLE_RATIO env var, defaults to 0.1
fn get_sampler() -> Sampler {
    // Get the sample ratio from the env var, default to 0.1
    let ratio_to_sample = std::env::var("TRACING_SAMPLE_RATIO")
        .unwrap_or_else(|_| "0.1".to_string())
        .parse::<f64>()
        .unwrap_or(0.1);

    // Create sampler based on
    // (1) parent => so if parent exists always sample
    // (2) if no parent, then the trace id ratio
    Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio_to_sample)))
}
