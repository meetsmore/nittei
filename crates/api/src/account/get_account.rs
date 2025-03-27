use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::get_account::APIResponse;
use nittei_infra::NitteiContext;

use crate::{error::NitteiError, shared::auth::protect_admin_route};

#[utoipa::path(
    get,
    tag = "Account",
    path = "/api/v1/account",
    summary = "Get the current account details"
)]
pub async fn get_account_controller(
    http_req: HttpRequest,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account_possibly_stale = protect_admin_route(&http_req, &ctx).await?;

    // Refetch the account, as the protect_admin_route uses a cached method
    // Meaning that the account could have been deleted in the meantime (or updated)
    let account = ctx
        .repos
        .accounts
        .find(&account_possibly_stale.id)
        .await
        .map_err(|_| NitteiError::InternalError)?
        .ok_or(NitteiError::NotFound("Account not found".to_string()))?;

    Ok(HttpResponse::Ok().json(APIResponse::new(account)))
}
