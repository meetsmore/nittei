use axum::{Extension, Json};
use nittei_api_structs::get_account::APIResponse;
use nittei_domain::Account;
use nittei_infra::NitteiContext;

use crate::error::NitteiError;

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
    Extension(ctx): Extension<NitteiContext>,
    Extension(account_possibly_stale): Extension<Account>,
) -> Result<Json<APIResponse>, NitteiError> {
    // Refetch the account, as the protect_admin_route uses a cached method
    // Meaning that the account could have been deleted in the meantime (or updated)
    let account = ctx
        .repos
        .accounts
        .find(&account_possibly_stale.id)
        .await
        .map_err(|_| NitteiError::InternalError)?
        .ok_or(NitteiError::NotFound("Account not found".to_string()))?;

    Ok(Json(APIResponse::new(account)))
}
