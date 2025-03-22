use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_datadog::{ApiVersion, DatadogPipelineBuilder};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{self, RandomIdGenerator, Sampler, SdkTracerProvider},
};
use tracing::warn;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber() -> anyhow::Result<()> {
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
        let observability_config = nittei_utils::config::APP_CONFIG.observability.as_ref();
        // In production, use the JSON format for logs
        let service_name = observability_config
            .and_then(|o| o.service_name.clone())
            .unwrap_or_else(|| "unknown service".to_string());
        let service_version = observability_config
            .and_then(|o| o.service_version.clone())
            .unwrap_or_else(|| "unknown version".to_string());
        let service_env = observability_config
            .and_then(|o| o.service_env.clone())
            .unwrap_or_else(|| "unknown env".to_string());

        // Set the global propagator to trace context propagator
        global::set_text_map_propagator(TraceContextPropagator::new());

        // Get the tracer - if no endpoint is provided, tracing will be disabled
        let tracer_provider =
            get_tracer_provider(service_name.clone(), service_version, service_env)?;

        // Create a telemetry layer if a tracer is available
        let telemetry_layer = tracer_provider.map(|tracer_provider| {
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer(service_name))
        });

        // Combine layers into a single subscriber
        if let Some(telemetry_layer) = telemetry_layer {
            let subscriber = Registry::default()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_current_span(false),
                )
                .with(telemetry_layer);

            // Set the global subscriber
            tracing::subscriber::set_global_default(subscriber)?;
        } else {
            // If no tracer is available, do not include telemetry layer
            let subscriber = Registry::default().with(env_filter).with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_current_span(false),
            );

            // Set the global subscriber
            tracing::subscriber::set_global_default(subscriber)?
        }
    }
    Ok(())
}

/// Get the tracer
fn get_tracer_provider(
    service_name: String,
    service_version: String,
    service_env: String,
) -> anyhow::Result<Option<SdkTracerProvider>> {
    let otlp_endpoint = nittei_utils::config::APP_CONFIG
        .observability
        .as_ref()
        .and_then(|o| o.otlp_tracing_endpoint.clone());
    let datadog_endpoint = nittei_utils::config::APP_CONFIG
        .observability
        .as_ref()
        .and_then(|o| o.datadog_tracing_endpoint.clone());

    if let Some(datadog_endpoint) = datadog_endpoint {
        Ok(Some(get_tracer_datadog(
            datadog_endpoint,
            service_name,
            service_version,
            service_env,
        )?))
    } else if let Some(otlp_endpoint) = otlp_endpoint {
        Ok(Some(get_tracer_otlp(
            otlp_endpoint,
            service_name,
            service_version,
            service_env,
        )?))
    } else {
        warn!(
            "No tracing endpoints provided (DATADOG_TRACING_ENDPOINT or OTLP_TRACING_ENDPOINT), tracing will be disabled"
        );
        Ok(None)
    }
}

/// Get the tracer based on the tracing endpoint
/// This is for the (unofficial) Datadog exporter
fn get_tracer_datadog(
    datadog_endpoint: String,
    service_name: String,
    service_version: String,
    service_env: String,
) -> anyhow::Result<SdkTracerProvider> {
    DatadogPipelineBuilder::default()
        .with_service_name(service_name)
        .with_version(service_version)
        .with_env(service_env)
        .with_api_version(ApiVersion::Version05)
        .with_agent_endpoint(datadog_endpoint)
        .with_trace_config(
            // Datadog lib is not yet adapted for the new way
            #[allow(deprecated)]
            trace::Config::default()
                .with_sampler(get_sampler())
                .with_id_generator(RandomIdGenerator::default()),
        )
        .install_batch()
        .map_err(|e| e.into())
}

/// Get the tracer based on the OTLP endpoint
/// This is for the OpenTelemetry Protocol (OTLP) exporter
fn get_tracer_otlp(
    otlp_endpoint: String,
    service_name: String,
    service_version: String,
    service_env: String,
) -> anyhow::Result<SdkTracerProvider> {
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpJson)
        .with_endpoint(otlp_endpoint)
        .build()?;

    // Then pass it into pipeline builder
    Ok(SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes(vec![
                    opentelemetry::KeyValue::new("service.name", service_name.clone()),
                    opentelemetry::KeyValue::new("service.version", service_version),
                    opentelemetry::KeyValue::new("deployment.environment", service_env),
                ])
                .build(),
        )
        .with_batch_exporter(otlp_exporter)
        .build())
}

/// Get the sampler to be used
/// This is a parent-based sampler, so if a parent exists, always sample
/// If there is no parent, then this is based on the TRACING_SAMPLE_RATIO env var, defaults to 0.1
fn get_sampler() -> Sampler {
    // Get the sample ratio from the env var, default to 0.1
    let ratio_to_sample = nittei_utils::config::APP_CONFIG
        .observability
        .as_ref()
        .and_then(|o| o.tracing_sample_rate)
        .unwrap_or(0.1);

    // Create sampler based on
    // (1) parent => so if parent exists always sample
    // (2) if no parent, then the trace id ratio
    Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio_to_sample)))
}
