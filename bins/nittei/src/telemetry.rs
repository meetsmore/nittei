use std::time::Duration;

use opentelemetry::{global, propagation::TextMapCompositePropagator, trace::TracerProvider};
use opentelemetry_datadog::{ApiVersion, DatadogPipelineBuilder, DatadogPropagator};
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
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
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        // Downgrade opentelemetry_sdk errors to warnings
        .add_directive("opentelemetry_sdk=warn".parse()?);

    // If the binary is compiled in debug mode (aka for development)
    // use the compact format for logs
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    } else {
        let observability_config = &nittei_utils::config::APP_CONFIG.observability;
        // In production, use the JSON format for logs
        let service_name = &observability_config.service_name;
        let service_version = &observability_config.service_version;
        let service_env = &observability_config.service_env;

        // Set the global propagator to trace context propagator
        let composite = TextMapCompositePropagator::new(vec![
            Box::new(TraceContextPropagator::new()),
            Box::new(DatadogPropagator::new()),
        ]);

        global::set_text_map_propagator(composite);

        // Get the tracer - if no endpoint is provided, tracing will be disabled
        let tracer_provider = get_tracer_provider(
            service_name.to_owned(),
            service_version.to_owned(),
            service_env.to_owned(),
        )?;

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
    let otlp_endpoint = &nittei_utils::config::APP_CONFIG
        .observability
        .otlp_tracing_endpoint;
    let datadog_endpoint = &nittei_utils::config::APP_CONFIG
        .observability
        .datadog_tracing_endpoint;

    if let Some(datadog_endpoint) = datadog_endpoint {
        Ok(Some(get_tracer_datadog(
            datadog_endpoint.to_owned(),
            service_name,
            service_version,
            service_env,
        )?))
    } else if let Some(otlp_endpoint) = otlp_endpoint {
        Ok(Some(get_tracer_otlp(
            otlp_endpoint.to_owned(),
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
    let mut config = trace::Config::default();
    config.sampler = Box::new(get_sampler());
    config.id_generator = Box::new(RandomIdGenerator::default());

    let http_client = get_http_client()?;

    DatadogPipelineBuilder::default()
        .with_http_client(http_client)
        .with_service_name(service_name)
        .with_version(service_version)
        .with_env(service_env)
        .with_api_version(ApiVersion::Version05)
        .with_agent_endpoint(datadog_endpoint)
        .with_trace_config(config)
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
    let http_client = get_http_client()?;
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_http_client(http_client)
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
        .with_sampler(get_sampler())
        .build())
}

/// Get the sampler to be used
/// This is a parent-based sampler, so if a parent exists, always sample
/// If there is no parent, then this is based on the TRACING_SAMPLE_RATIO env var, defaults to 0.1
fn get_sampler() -> Sampler {
    // Get the sample ratio from the env var, default is 0.1
    let ratio_to_sample = nittei_utils::config::APP_CONFIG
        .observability
        .tracing_sample_rate;

    // If tracing is disabled, return an always off sampler
    if nittei_utils::config::APP_CONFIG
        .observability
        .disable_tracing
    {
        return Sampler::AlwaysOff;
    }

    // Create sampler based on
    // (1) parent => so if parent exists always sample
    // (2) if no parent, then the trace id ratio
    Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio_to_sample)))
}

/// Get the HTTP client to be used
/// This is used to send traces to the tracing endpoint
fn get_http_client() -> anyhow::Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client for telemetry: {}", e))
}
