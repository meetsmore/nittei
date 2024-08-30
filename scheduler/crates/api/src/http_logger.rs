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
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
