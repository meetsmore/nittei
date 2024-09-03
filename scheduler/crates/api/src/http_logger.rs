use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use tracing::{Level, Span};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};

pub struct NitteiTracingRootSpanBuilder;

impl RootSpanBuilder for NitteiTracingRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        // Ignore healthcheck endpoint
        let level = if request.path() == "/api/v1/healthcheck" {
            Level::DEBUG
        } else {
            Level::INFO
        };
        tracing_actix_web::root_span!(level = level, request)
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        // Log the outcome of the request
        if let Ok(response) = outcome {
            let status_code = response.status().as_u16();
            let method = response.request().method().to_string();
            let path = response.request().path().to_string();

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
        } else if let Err(err) = outcome {
            // Fallback in case we can't retrieve the request from the span
            tracing::error!(
                status_code = 500,
                error = %err,
                "HTTP request resulted in an error, but request details are missing"
            );
        }

        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
