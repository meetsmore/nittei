use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::get_me::*;
use nittei_infra::NitteiContext;

use crate::{error::NitteiError, shared::auth::protect_route};

pub async fn get_me_controller(
    http_req: HttpRequest,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, _) = protect_route(&http_req, &ctx).await?;

    Ok(HttpResponse::Ok().json(APIResponse::new(user)))
}
