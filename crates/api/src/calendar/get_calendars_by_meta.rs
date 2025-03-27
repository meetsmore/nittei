use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::get_calendars_by_meta::*;
use nittei_domain::Metadata;
use nittei_infra::{MetadataFindQuery, NitteiContext};

use crate::{error::NitteiError, shared::auth::protect_admin_route};

#[utoipa::path(
    get,
    tag = "Calendar",
    path = "/api/v1/calendar/meta",
    summary = "Get calendars by metadata (admin only)",
    security(
        ("api_key" = [])
    ),
    params(
        ("key" = String, Query, description = "The key of the metadata to search for"),
        ("value" = String, Query, description = "The value of the metadata to search for"),
        ("skip" = Option<usize>, Query, description = "The number of calendars to skip"),
        ("limit" = Option<usize>, Query, description = "The number of calendars to return"),
    ),
    responses(
        (status = 200, body = GetCalendarsByMetaAPIResponse)
    )
)]
pub async fn get_calendars_by_meta_controller(
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
    let calendars = ctx
        .repos
        .calendars
        .find_by_metadata(query)
        .await
        .map_err(|_| NitteiError::InternalError)?;
    Ok(HttpResponse::Ok().json(GetCalendarsByMetaAPIResponse::new(calendars)))
}
