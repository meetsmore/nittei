use std::sync::Arc;

use chrono::{DateTime, Utc};
// Re-export API structs
pub use multiple_freebusy::{MultipleFreeBusyAPIResponse, MultipleFreeBusyRequestBody};
use nittei_api_structs::*;
use nittei_domain::IntegrationProvider;
use reqwest::StatusCode;

use crate::{APIResponse, BaseClient, ID, shared::MetadataFindInput};

#[derive(Clone)]
pub struct UserClient {
    base: Arc<BaseClient>,
}
pub struct UpdateUserInput {
    pub user_id: ID,
    pub metadata: Option<serde_json::Value>,
    pub external_id: Option<String>,
}

pub type CreateUserInput = create_user::CreateUserRequestBody;

pub struct GetUserFreeBusyInput {
    pub user_id: ID,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub calendar_ids: Option<Vec<ID>>,
}

impl From<GetUserFreeBusyInput> for Vec<(String, String)> {
    fn from(inp: GetUserFreeBusyInput) -> Self {
        let mut query = vec![
            ("startTime".to_string(), inp.start_time.to_rfc3339()),
            ("endTime".to_string(), inp.end_time.to_rfc3339()),
        ];
        if let Some(calendar_ids) = inp.calendar_ids {
            let calendar_ids = calendar_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            query.push(("calendarIds".to_string(), calendar_ids));
        }

        query
    }
}

pub struct OAuthInput {
    pub user_id: ID,
    pub code: String,
    pub provider: IntegrationProvider,
}

pub struct RemoveUserIntegrationInput {
    pub user_id: ID,
    pub provider: IntegrationProvider,
}

impl UserClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn create(&self, input: CreateUserInput) -> APIResponse<create_user::APIResponse> {
        self.base
            .post(input, "user".into(), StatusCode::CREATED)
            .await
    }

    pub async fn get(&self, user_id: ID) -> APIResponse<get_user::APIResponse> {
        self.base
            .get(format!("user/{user_id}"), None, StatusCode::OK)
            .await
    }

    pub async fn get_by_external_id(
        &self,
        external_id: String,
    ) -> APIResponse<get_user::APIResponse> {
        self.base
            .get(
                format!("user/external_id/{external_id}"),
                None,
                StatusCode::OK,
            )
            .await
    }

    pub async fn delete(&self, user_id: ID) -> APIResponse<delete_user::APIResponse> {
        self.base
            .delete(format!("user/{user_id}"), StatusCode::OK)
            .await
    }

    pub async fn update(&self, input: UpdateUserInput) -> APIResponse<update_user::APIResponse> {
        let body = update_user::UpdateUserRequestBody {
            external_id: input.external_id,
            metadata: input.metadata,
        };
        self.base
            .put(body, format!("user/{}", input.user_id), StatusCode::OK)
            .await
    }

    pub async fn free_busy(
        &self,
        query: GetUserFreeBusyInput,
    ) -> APIResponse<get_user_freebusy::GetUserFreeBusyAPIResponse> {
        let user_id = query.user_id.clone();
        self.base
            .get(
                format!("user/{user_id}/freebusy"),
                Some(query.into()),
                StatusCode::OK,
            )
            .await
    }

    /// Get free/busy information for multiple users between a time range
    ///
    /// Response will contain a hashmap of user's UUID with their free/busy information
    pub async fn multiple_users_free_busy(
        &self,
        body: MultipleFreeBusyRequestBody,
    ) -> APIResponse<MultipleFreeBusyAPIResponse> {
        self.base
            .post(body, "user/freebusy".to_string(), StatusCode::OK)
            .await
    }

    pub async fn oauth(&self, input: OAuthInput) -> APIResponse<oauth_integration::APIResponse> {
        let user_id = input.user_id.clone();
        let body = oauth_integration::OAuthIntegrationRequestBody {
            code: input.code,
            provider: input.provider,
        };
        self.base
            .post(body, format!("user/{user_id}/oauth"), StatusCode::OK)
            .await
    }

    pub async fn remove_integration(
        &self,
        input: RemoveUserIntegrationInput,
    ) -> APIResponse<remove_integration::APIResponse> {
        let provider: String = input.provider.clone().into();
        self.base
            .delete(
                format!("user/{}/oauth/{}", input.user_id, provider),
                StatusCode::OK,
            )
            .await
    }

    pub async fn get_by_meta(
        &self,
        input: MetadataFindInput,
    ) -> APIResponse<get_users_by_meta::GetUsersByMetaAPIResponse> {
        self.base
            .get(
                "user/meta".to_string(),
                Some(input.to_query()),
                StatusCode::OK,
            )
            .await
    }
}
