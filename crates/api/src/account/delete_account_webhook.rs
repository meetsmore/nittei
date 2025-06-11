use axum::{Extension, Json};
use nittei_api_structs::delete_account_webhook::APIResponse;
use nittei_domain::Account;
use nittei_infra::NitteiContext;

use super::set_account_webhook::SetAccountWebhookUseCase;
use crate::{error::NitteiError, shared::usecase::execute};

#[utoipa::path(
    delete,
    tag = "Account",
    path = "/api/v1/account/webhook",
    summary = "Delete the webhook for an account",
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = APIResponse)
    )
)]
pub async fn delete_account_webhook_controller(
    Extension(account): Extension<Account>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let usecase = SetAccountWebhookUseCase {
        account,
        webhook_url: None,
    };

    execute(usecase, &ctx)
        .await
        .map(|account| Json(APIResponse::new(account)))
        .map_err(NitteiError::from)
}
