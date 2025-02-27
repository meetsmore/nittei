use axum::{extract::State, http::HeaderMap, Json};
use nittei_api_structs::get_account::APIResponse;
use nittei_infra::NitteiContext;

use crate::{error::NitteiError, shared::auth::protect_account_route};

pub async fn get_account_controller(
    headers: HeaderMap,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_account_route(&headers, &ctx).await?;

    Ok(Json(APIResponse::new(account)))
}
