use axum::{Extension, Json, extract::Query, http::HeaderMap};
use nittei_api_structs::get_users_by_meta::*;
use nittei_domain::Metadata;
use nittei_infra::{MetadataFindQuery, NitteiContext};

use crate::{error::NitteiError, shared::auth::protect_admin_route};

#[utoipa::path(
    get,
    tag = "User",
    path = "/api/v1/user/meta",
    summary = "Get users by metadata",
    params(
        ("key" = String, Query, description = "The key of the metadata to search for"),
        ("value" = String, Query, description = "The value of the metadata to search for"),
        ("skip" = Option<usize>, Query, description = "The number of users to skip"),
        ("limit" = Option<usize>, Query, description = "The number of users to return"),
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, body = GetUsersByMetaAPIResponse)
    )
)]
pub async fn get_users_by_meta_controller(
    headers: HeaderMap,
    query_params: Query<QueryParams>,
    Extension(ctx): Extension<NitteiContext>,
) -> Result<Json<GetUsersByMetaAPIResponse>, NitteiError> {
    let account = protect_admin_route(&headers, &ctx).await?;

    let query = MetadataFindQuery {
        account_id: account.id,
        metadata: Metadata::new_kv(query_params.0.key, query_params.0.value),
        limit: query_params.0.limit.unwrap_or(20),
        skip: query_params.0.skip.unwrap_or(0),
    };
    let users = ctx
        .repos
        .users
        .find_by_metadata(query)
        .await
        .map_err(|_| NitteiError::InternalError)?;
    Ok(Json(GetUsersByMetaAPIResponse::new(users)))
}
