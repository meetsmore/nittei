use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::get_account::APIResponse;
use nittei_infra::NitteiContext;

use crate::{error::NitteiError, shared::auth::protect_account_route};

pub async fn get_account_controller(
    http_req: HttpRequest,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    Ok(HttpResponse::Ok().json(APIResponse::new(account)))
}
