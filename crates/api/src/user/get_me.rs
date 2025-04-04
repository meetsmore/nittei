use axum::{Extension, Json, http::HeaderMap};
use nittei_api_structs::get_me::*;
use nittei_infra::NitteiContext;

use crate::{error::NitteiError, shared::auth::protect_route};

#[utoipa::path(
    get,
    tag = "User",
    path = "/api/v1/me",
    summary = "Get the current user",
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn get_me_controller(
    headers: HeaderMap,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let (user, _) = protect_route(&headers, &ctx).await?;

    Ok(Json(APIResponse::new(user)))
}
