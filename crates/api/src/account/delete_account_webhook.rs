use axum::{extract::State, http::HeaderMap, Json};
use nittei_api_structs::delete_account_webhook::APIResponse;
use nittei_infra::NitteiContext;

use super::set_account_webhook::SetAccountWebhookUseCase;
use crate::{
    error::NitteiError,
    shared::{auth::protect_account_route, usecase::execute},
};

pub async fn delete_account_webhook_controller(
    headers: HeaderMap,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_account_route(&headers, &ctx).await?;

    let usecase = SetAccountWebhookUseCase {
        account,
        webhook_url: None,
    };

    execute(usecase, &ctx)
        .await
        .map(|account| Json(APIResponse::new(account)))
        .map_err(NitteiError::from)
}
