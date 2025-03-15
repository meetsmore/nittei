use axum::{
    Extension,
    Json,
    extract::Query,
    http::HeaderMap,
};
use nittei_api_structs::get_schedules_by_meta::*;
use nittei_domain::Metadata;
use nittei_infra::{MetadataFindQuery, NitteiContext};

use crate::{error::NitteiError, shared::auth::protect_admin_route};

pub async fn get_schedules_by_meta_controller(
    headers: HeaderMap,
    query_params: Query<QueryParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<APIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

    let query = MetadataFindQuery {
        account_id: account.id,
        metadata: Metadata::new_kv(query_params.0.key, query_params.0.value),
        limit: query_params.0.limit.unwrap_or(20),
        skip: query_params.0.skip.unwrap_or(0),
    };
    let schedules = ctx
        .repos
        .schedules
        .find_by_metadata(query)
        .await
        .map_err(|_| NitteiError::InternalError)?;
    Ok(Json(APIResponse::new(schedules)))
}
