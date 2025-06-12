use axum::{Extension, Json};
use nittei_api_structs::get_me::*;
use nittei_domain::User;

use crate::{error::NitteiError, shared::auth::Policy};

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
    Extension((user, _policy)): Extension<(User, Policy)>,
) -> Result<Json<APIResponse>, NitteiError> {
    Ok(Json(APIResponse::new(user)))
}
