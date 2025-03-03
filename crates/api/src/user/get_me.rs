use axum::{Json, extract::State, http::HeaderMap};
use nittei_api_structs::get_me::*;
use nittei_infra::NitteiContext;

use crate::{error::NitteiError, shared::auth::protect_route};

pub async fn get_me_controller(
    headers: HeaderMap,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let (user, _) = protect_route(&headers, &ctx).await?;

    Ok(Json(APIResponse::new(user)))
}
