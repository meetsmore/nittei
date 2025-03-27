use nittei_domain::{ID, User};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use validator::Validate;

use crate::dtos::UserDTO;

/// User response object
#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserResponse {
    /// User retrieved
    pub user: UserDTO,
}

impl UserResponse {
    pub fn new(user: User) -> Self {
        Self {
            user: UserDTO::new(user),
        }
    }
}

pub mod get_me {
    use super::*;

    pub type APIResponse = UserResponse;
}

pub mod get_user_by_external_id {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub external_id: String,
    }

    pub type APIResponse = UserResponse;
}

pub mod create_user {
    use super::*;

    /// Request body for creating a user
    #[derive(Debug, Deserialize, Serialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateUserRequestBody")]
    pub struct RequestBody {
        /// Optional metadata (e.g. {"key": "value"})
        #[serde(default)]
        #[ts(optional)]
        pub metadata: Option<serde_json::Value>,

        /// Optional external ID (e.g. the ID of the user in an external system)
        #[serde(default)]
        #[ts(optional)]
        pub external_id: Option<String>,

        /// Optional user ID
        /// If not provided, a new UUID will be generated
        /// This is useful for external applications that need to link Nittei's users to their own data models
        #[serde(default)]
        #[ts(optional)]
        pub user_id: Option<ID>,
    }

    pub type APIResponse = UserResponse;
}

pub mod oauth_integration {
    use nittei_domain::IntegrationProvider;

    use super::*;

    /// Request body for creating an OAuth integration
    #[derive(Debug, Deserialize, Serialize, Validate, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "OAuthIntegrationRequestBody")]
    pub struct RequestBody {
        /// OAuth code
        #[validate(length(min = 1))]
        pub code: String,

        /// Integration provider
        /// E.g. "Google", "Outlook"
        pub provider: IntegrationProvider,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod remove_integration {
    use nittei_domain::IntegrationProvider;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PathParams {
        pub provider: IntegrationProvider,
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod oauth_outlook {
    use super::*;

    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "OAuthOutlookRequestBody")]
    pub struct RequestBody {
        pub code: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod update_user {
    use super::*;

    /// Request body for updating a user
    #[derive(Debug, Deserialize, Serialize, TS, ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "UpdateUserRequestBody")]
    pub struct RequestBody {
        /// Optional external ID (e.g. the ID of the user in an external system)
        #[serde(default)]
        #[ts(optional)]
        pub external_id: Option<String>,

        /// Optional metadata (e.g. {"key": "value"})
        #[serde(default)]
        #[ts(optional)]
        pub metadata: Option<serde_json::Value>,
    }

    #[derive(Debug, Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod delete_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod get_user {
    use super::*;

    #[derive(Deserialize)]
    pub struct PathParams {
        pub user_id: ID,
    }

    pub type APIResponse = UserResponse;
}

pub mod get_users_by_meta {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryParams {
        pub key: String,
        pub value: String,
        #[serde(default)]
        pub skip: Option<usize>,
        pub limit: Option<usize>,
    }

    /// API response for getting users by metadata
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "GetUsersByMetaAPIResponse")]
    pub struct APIResponse {
        /// List of users matching the metadata query
        pub users: Vec<UserDTO>,
    }

    impl APIResponse {
        pub fn new(users: Vec<User>) -> Self {
            Self {
                users: users.into_iter().map(UserDTO::new).collect(),
            }
        }
    }
}
