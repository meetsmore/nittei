use axum::{
    body::Body,
    http::{Request, Response},
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, Span};

/// Custom root span builder (tracing) for Axum
pub struct NitteiTracingRootSpanBuilder;

impl NitteiTracingRootSpanBuilder {
    /// Create a new root span for the incoming request
    pub fn on_request_start<B>(request: &Request<B>) -> Span {
        // Ignore healthcheck endpoint
        let level = if request.uri().path() == "/api/v1/healthcheck" {
            Level::DEBUG
        } else {
            Level::INFO
        };
        tracing::span!(level, "request", method = %request.method(), uri = %request.uri())
    }

    /// End the root span for the incoming request
    pub fn on_request_end<B>(span: Span, response: &Response<B>) {
        // Log the outcome of the request
        log_request(response);

        span.record("status", &tracing::field::display(response.status()));
    }
}

/// Log the outcome of the request
fn log_request<B>(response: &Response<B>) {
    // Log the outcome of the request
    let status_code = response.status().as_u16();
    let method = response.request().method().to_string();
    let path = response.request().uri().path().to_string();

    // Ignore healthcheck endpoint
    if path == "/api/v1/healthcheck" {
        return;
    }

    // Log with custom fields in JSON format
    let message = format!("{} {} => {}", method, path, status_code);

    if status_code >= 500 {
        tracing::error!(
            method = method,
            path = path,
            status_code = status_code,
            message,
        );
    } else if status_code >= 400 {
        tracing::warn!(
            method = method,
            path = path,
            status_code = status_code,
            message,
        );
    } else {
        tracing::info!(
            method = method,
            path = path,
            status_code = status_code,
            message,
        );
    };
}

/// Create a TraceLayer for Axum with custom root span builder
pub fn create_trace_layer() -> TraceLayer<DefaultMakeSpan, DefaultOnResponse> {
    TraceLayer::new_for_http()
        .make_span_with(NitteiTracingRootSpanBuilder::on_request_start)
        .on_response(NitteiTracingRootSpanBuilder::on_request_end)
}
