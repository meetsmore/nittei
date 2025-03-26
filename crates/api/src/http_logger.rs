use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Request, Response, Uri},
    middleware::Next,
    response::Response as AxumResponse,
};
use opentelemetry::{
    global::{self},
    trace::{Span as OtelSpan, Tracer},
};
use opentelemetry_http::HeaderExtractor;
use tower_http::trace::{MakeSpan, OnFailure, OnResponse};
use tracing::{Span, field::Empty};

/// Metadata for the request
/// Used for logging and tracing
#[derive(Clone)]
struct RequestMetadata {
    pub method: String,
    pub uri: Uri,
    pub matched_path: String,
}

/// Middleware for storing metadata for logging and tracing
pub async fn metadata_middleware(request: Request<Body>, next: Next) -> AxumResponse {
    let uri = request.uri().clone();
    let method = request.method().to_string();

    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|r| r.as_str().to_string())
        .unwrap_or_default();

    let mut response = next.run(request).await;

    response.extensions_mut().insert(RequestMetadata {
        method,
        uri,
        matched_path,
    });

    response
}

/// Custom span builder for Axum requests
#[derive(Clone)]
pub struct NitteiTracingSpanBuilder {}

impl<B> MakeSpan<B> for NitteiTracingSpanBuilder {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let http_method = request.method().to_string();
        let path = request.uri().path();
        let query = request.uri().query().unwrap_or_default();
        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(|r| r.as_str().to_string())
            .unwrap_or_default();

        if path == "/api/v1/healthcheck" {
            return Span::none();
        }

        // Extract trace context from request headers
        let parent_cx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(request.headers()))
        });

        // Create a span within the extracted context using the global tracer
        let otel_span = global::tracer("http").start_with_context("http.request", &parent_cx);

        // Create a tracing span that wraps the OpenTelemetry span
        tracing::info_span!(
            "http.request",
            http.request.method = %http_method,
            http.route = Empty, // to set by router of "webframework" after
            http.status_code = Empty, // to set on response (datadog attribute)
            url.path = %path,
            url.query = %query,
            url.scheme = request.uri().scheme_str().unwrap_or("http"),
            otel.name = %format!("{} {}", http_method, path),
            otel.kind = ?opentelemetry::trace::SpanKind::Server,
            otel.status_code = Empty, // to set on response
            resource.name = %matched_path,
            request_id = Empty, // to set
            exception.message = Empty, // to set on response
            "span.type" = "web",
            level = Empty, // will be set in on_response based on status code
            trace_id = %otel_span.span_context().trace_id().to_string(),
            span_id = %otel_span.span_context().span_id().to_string(),
        )
    }
}

/// Custom response handler for Axum tracing
#[derive(Clone)]
pub struct NitteiTracingOnResponse {}

impl<B> OnResponse<B> for NitteiTracingOnResponse {
    fn on_response(self, response: &Response<B>, latency: std::time::Duration, span: &Span) {
        let status_code = response.status().as_u16();

        let metadata = response.extensions().get::<RequestMetadata>();
        let method = metadata.map(|m| m.method.clone()).unwrap_or_default();
        let uri = metadata.map(|m| m.uri.clone()).unwrap_or_default();
        let uri_string = uri.to_string();
        let path = uri.path();
        let matched_path = metadata.map(|m| m.matched_path.clone()).unwrap_or_default();

        // Set OpenTelemetry status code and level based on status code
        let (otel_status, level) = if status_code >= 500 {
            ("error", tracing::Level::ERROR)
        } else if status_code >= 400 {
            ("warn", tracing::Level::WARN)
        } else {
            ("ok", tracing::Level::INFO)
        };

        // Exclude health check from logging
        if path == "/api/v1/healthcheck" && status_code == 200 {
            return;
        }

        // Update span attributes
        span.record("http.status_code", status_code);
        span.record("http.route", matched_path.clone());
        span.record("duration", latency.as_nanos() as f64 / 1_000_000.0); // Convert to milliseconds
        span.record("resource.name", matched_path.clone());
        span.record("otel.status_code", otel_status);
        // Adjust span level dynamically
        tracing::span::Span::current().record("level", tracing::field::display(level));

        let message = format!(
            "{} {} {} {}ns",
            method,
            uri_string,
            status_code,
            latency.as_nanos()
        );

        // Log with appropriate level
        match otel_status {
            "error" => tracing::error!(
                parent: span,
                method = %method,
                path = %path,
                status_code = %status_code,
                duration = %latency.as_nanos(),
                route = %matched_path,
                message
            ),
            "warn" => tracing::warn!(
                parent: span,
                method = %method,
                path = %path,
                status_code = %status_code,
                duration = %latency.as_nanos(),
                route = %matched_path,
                message
            ),
            "info" => tracing::info!(
                parent: span,
                method = %method,
                path = %path,
                status_code = %status_code,
                duration = %latency.as_nanos(),
                route = %matched_path,
                message
            ),
            _ => tracing::info!(
                parent: span,
                method = %method,
                path = %path,
                status_code = %status_code,
                duration = %latency.as_nanos(),
                route = %matched_path,
                message
            ),
        }
    }
}

/// Custom failure handler for Axum tracing
#[derive(Clone)]
pub struct NitteiTracingOnFailure {}

impl<E: std::fmt::Debug> OnFailure<E> for NitteiTracingOnFailure {
    fn on_failure(&mut self, error: E, latency: std::time::Duration, span: &Span) {
        span.record("level", "error");
        span.record("otel.status_code", "error");
        span.record("duration", latency.as_nanos());
        span.record("exception.type", std::any::type_name_of_val(&error));
        span.record("exception.message", format!("{:?}", error));

        tracing::error!(
            parent: span,
            error = ?error,
            duration = %latency.as_nanos(),
            "Request failed"
        );
    }
}
