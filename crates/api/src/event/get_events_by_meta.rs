use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::get_events_by_meta::*;
use nittei_domain::Metadata;
use nittei_infra::{MetadataFindQuery, NitteiContext};

use crate::{error::NitteiError, shared::auth::protect_admin_route};

#[utoipa::path(
    get,
    tag = "Event",
    path = "/api/v1/events/meta",
    summary = "Get events by metadata (admin only)",
    params(
        ("key" = String, Query, description = "The key of the metadata to search for"),
        ("value" = String, Query, description = "The value of the metadata to search for"),
        ("skip" = Option<usize>, Query, description = "The number of events to skip"),
        ("limit" = Option<usize>, Query, description = "The number of events to return"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = GetEventsByMetaAPIResponse)
    )
)]
pub async fn get_events_by_meta_controller(
    http_req: HttpRequest,
    query_params: web::Query<QueryParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

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
        .map_err(|_| NitteiError::InternalError)?;
    Ok(HttpResponse::Ok().json(GetEventsByMetaAPIResponse::new(events)))
}
