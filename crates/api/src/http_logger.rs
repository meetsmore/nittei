use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Request, Response, Uri},
    middleware::Next,
    response::Response as AxumResponse,
};
use tower_http::trace::{MakeSpan, OnFailure, OnResponse};
use tracing::{Span, field::Empty};

#[derive(Clone)]
struct RequestMetadata {
    pub method: String,
    pub uri: Uri,
    pub matched_path: String,
}

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

        let span = tracing::trace_span!(
            "HTTP request",
            http.request.method = %http_method,
            http.route = Empty, // to set by router of "webframework" after
            http.client.address = Empty, //%$request.connection_info().realip_remote_addr().unwrap_or(""),
            http.response.status_code = Empty, // to set on response
            http.status_code = Empty, // to set on response (datadog attribute)
            url.path = request.uri().path(),
            url.query = request.uri().query(),
            // url.scheme = url_scheme(request.uri()),
            otel.name = %http_method, // to set by router of "webframework" after
            otel.kind = ?opentelemetry::trace::SpanKind::Server,
            otel.status_code = Empty, // to set on response
            trace_id = Empty, // to set on response
            request_id = Empty, // to set
            exception.message = Empty, // to set on response
            "span.type" = "web", // non-official open-telemetry key, only supported by Datadog
        );

        // Exclude health check from tracing
        if request.uri().path() == "/api/v1/healthcheck" {
            Span::none()
        } else {
            span
        }
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
        let path = uri.path();
        let matched_path = metadata.map(|m| m.matched_path.clone()).unwrap_or_default();

        // Log with custom fields in JSON format
        let message = format!(
            "{} {} {} {}ns",
            method,
            path,
            status_code,
            latency.as_nanos()
        );

        span.record("status_code", status_code);
        span.record("latency", latency.as_nanos());
        span.record("message", message.clone());
        span.record("http.route", matched_path);

        if status_code >= 500 {
            span.record("level", "error");
            tracing::error!(parent: span, method = method, path = path, message = message.as_str());
        } else if status_code >= 400 {
            span.record("level", "warn");
            tracing::warn!(parent: span, method = method, path = path, message = message.as_str());
        } else {
            span.record("level", "info");
            tracing::info!(parent: span, method = method, path = path, message = message.as_str());
        }
    }
}

/// Custom failure handler for Axum tracing
#[derive(Clone)]
pub struct NitteiTracingOnFailure {}

impl<E: std::fmt::Debug> OnFailure<E> for NitteiTracingOnFailure {
    fn on_failure(&mut self, error: E, _latency: std::time::Duration, span: &Span) {
        tracing::error!(
            parent: span,
            error = ?error,
            "Request failed"
        );
    }
}
