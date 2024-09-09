use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_api_structs::get_events_by_meta::*;
use nettu_scheduler_domain::Metadata;
use nettu_scheduler_infra::{MetadataFindQuery, NettuContext};

use crate::{error::NettuError, shared::auth::protect_account_route};

pub async fn get_events_by_meta_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let query = MetadataFindQuery {
        account_id: account.id,
        metadata: Metadata::new_kv(query_params.0.key, query_params.0.value),
        limit: query_params.0.limit.unwrap_or(20),
        skip: query_params.0.skip.unwrap_or(0),
    };
    let events = ctx
        .repos
        .events
        .find_by_metadata(query)
        .await
        .map_err(|_| NettuError::InternalError)?;
    Ok(HttpResponse::Ok().json(APIResponse::new(events)))
}
