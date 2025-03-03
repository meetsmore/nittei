use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use nittei_api_structs::get_events_by_meta::*;
use nittei_domain::Metadata;
use nittei_infra::{MetadataFindQuery, NitteiContext};

use crate::{error::NitteiError, shared::auth::protect_admin_route};

pub async fn get_events_by_meta_controller(
    headers: HeaderMap,
    query_params: Query<QueryParams>,
    State(ctx): State<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

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
    Ok(Json(APIResponse::new(events)))
}
