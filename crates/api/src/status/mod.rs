use axum::{Extension, Json, http::StatusCode, routing::get};
use nittei_api_structs::get_service_health::*;
use nittei_infra::NitteiContext;
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

/// Configure the routes for the status module
pub fn configure_routes() -> OpenApiRouter {
    OpenApiRouter::new()
        // Get the health status of the service
        .route("/healthcheck", get(status))
}
