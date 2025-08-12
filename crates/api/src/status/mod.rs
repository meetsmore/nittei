use axum::{Extension, Json, http::StatusCode, routing::get};
use nittei_api_structs::get_service_health::*;
use nittei_infra::{NitteiContext, metrics::INFRA_REGISTRY};
use prometheus::TextEncoder;
use utoipa_axum::router::OpenApiRouter;

/// Get the status of the service
async fn status(Extension(ctx): Extension<NitteiContext>) -> (StatusCode, Json<APIResponse>) {
    match ctx.repos.status.check_connection().await {
        Ok(_) => (
            StatusCode::OK,
            Json(APIResponse {
                message: "Ok!\r\n".into(),
            }),
        ),
        Err(e) => {
            tracing::error!("[status] Error checking connection: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(APIResponse {
                    message: "Internal Server Error".into(),
                }),
            )
        }
    }
}

/// Get the metrics of the service
async fn metrics() -> (StatusCode, String) {
    let encoder = TextEncoder::new();

    // Gather metrics from both the default registry and our custom registry
    let mut metric_families = prometheus::gather();
    let infra_metrics = INFRA_REGISTRY.gather();
    metric_families.extend(infra_metrics);

    let body = if let Ok(body) = encoder.encode_to_string(&metric_families) {
        body
    } else {
        "Error encoding metrics".into()
    };

    (StatusCode::OK, body)
}

/// Configure the routes for the status module
pub fn configure_routes() -> OpenApiRouter {
    OpenApiRouter::new()
        // Get the health status of the service
        .route("/healthcheck", get(status))
        .route("/metrics", get(metrics))
}
