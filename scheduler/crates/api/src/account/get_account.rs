use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_account::APIResponse;
use nettu_scheduler_infra::NettuContext;

use crate::{error::NettuError, shared::auth::protect_account_route};

pub async fn get_account_controller(
    http_req: HttpRequest,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    Ok(HttpResponse::Ok().json(APIResponse::new(account)))
}
