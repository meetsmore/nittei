use axum::{Extension, Json, http::HeaderMap};
use nittei_api_structs::get_account::APIResponse;
use nittei_infra::NitteiContext;

use crate::{error::NitteiError, shared::auth::protect_admin_route};

#[utoipa::path(
    get,
    tag = "Account",
    path = "/api/v1/account",
    summary = "Get the current account details",
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = APIResponse),
    )
)]
pub async fn get_account_controller(
    headers: HeaderMap,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

    Ok(Json(APIResponse::new(account)))
}
