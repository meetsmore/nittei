use std::sync::Arc;

use chrono::{DateTime, Utc};
// Re-export API structs
pub use multiple_freebusy::{
    APIResponse as MultipleFreeBusyAPIResponse,
    RequestBody as MultipleFreeBusyRequestBody,
};
use nettu_scheduler_api_structs::*;
use nettu_scheduler_domain::{IntegrationProvider, Metadata};
use reqwest::StatusCode;

use crate::{shared::MetadataFindInput, APIResponse, BaseClient, ID};

#[derive(Clone)]
pub struct UserClient {
    base: Arc<BaseClient>,
}
pub struct UpdateUserInput {
    pub user_id: ID,
    pub metadata: Option<Metadata>,
}

pub type CreateUserInput = create_user::RequestBody;

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
            .get(format!("user/{}", user_id), None, StatusCode::OK)
            .await
    }

    pub async fn delete(&self, user_id: ID) -> APIResponse<delete_user::APIResponse> {
        self.base
            .delete(format!("user/{}", user_id), StatusCode::OK)
            .await
    }

    pub async fn update(&self, input: UpdateUserInput) -> APIResponse<update_user::APIResponse> {
        let body = update_user::RequestBody {
            metadata: input.metadata,
        };
        self.base
            .put(body, format!("user/{}", input.user_id), StatusCode::OK)
            .await
    }

    pub async fn free_busy(
        &self,
        query: GetUserFreeBusyInput,
    ) -> APIResponse<get_user_freebusy::APIResponse> {
        let user_id = query.user_id.clone();
        self.base
            .get(
                format!("user/{}/freebusy", user_id),
                Some(query.into()),
                StatusCode::OK,
            )
            .await
    }

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
        let body = oauth_integration::RequestBody {
            code: input.code,
            provider: input.provider,
        };
        self.base
            .post(body, format!("user/{}/oauth", user_id), StatusCode::OK)
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
    ) -> APIResponse<get_users_by_meta::APIResponse> {
        self.base
            .get(
                "user/meta".to_string(),
                Some(input.to_query()),
                StatusCode::OK,
            )
            .await
    }
}
