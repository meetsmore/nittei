use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::delete_account_webhook::APIResponse;
use nittei_infra::NitteiContext;

use super::set_account_webhook::SetAccountWebhookUseCase;
use crate::{
    error::NitteiError,
    shared::{auth::protect_admin_route, usecase::execute},
};

pub async fn delete_account_webhook_controller(
    http_req: HttpRequest,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let usecase = SetAccountWebhookUseCase {
        account,
        webhook_url: None,
    };

    execute(usecase, &ctx)
        .await
        .map(|account| HttpResponse::Ok().json(APIResponse::new(account)))
        .map_err(NitteiError::from)
}
