use axum::{Extension, Json, http::StatusCode, routing::get};
use nittei_api_structs::get_service_health::*;
use nittei_infra::{NitteiContext, metrics::INFRA_REGISTRY};
use prometheus::TextEncoder;
use utoipa_axum::router::OpenApiRouter;

/// Liveness probe — confirms the process is running.
/// k8s restarts the pod if this returns a non-2xx status.
/// No external dependency checks: if the process can respond, it is alive.
async fn liveness() -> (StatusCode, Json<APIResponse>) {
    (
        StatusCode::OK,
        Json(APIResponse {
            message: "Ok!\r\n".into(),
        }),
    )
}

/// Readiness probe — confirms the service can handle traffic.
/// k8s removes the pod from the load balancer (but does NOT restart it) when this fails.
/// Checks DB connectivity so traffic is only routed when dependencies are reachable.
async fn readiness(
    Extension(ctx): Extension<NitteiContext>,
) -> (StatusCode, Json<APIResponse>) {
    match ctx.repos.status.check_connection().await {
        Ok(_) => (
            StatusCode::OK,
            Json(APIResponse {
                message: "Ok!\r\n".into(),
            }),
        ),
        Err(e) => {
            tracing::error!("[status] Readiness check failed: {:?}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(APIResponse {
                    message: "Service Unavailable".into(),
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
        // Liveness probe: is the process alive?
        .route("/healthz/live", get(liveness))
        // Readiness probe: is the service ready to serve traffic?
        .route("/healthz/ready", get(readiness))
        .route("/metrics", get(metrics))
}
