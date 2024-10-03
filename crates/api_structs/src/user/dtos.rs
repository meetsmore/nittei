use nittei_domain::{Metadata, User, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// User object
#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserDTO {
    /// UUID of the user
    pub id: ID,
    /// Metadata (e.g. {"key": "value"})
    #[ts(type = "Record<string, string | number | boolean>")]
    pub metadata: Metadata,
}

impl UserDTO {
    pub fn new(user: User) -> Self {
        Self {
            id: user.id,
            metadata: user.metadata,
        }
    }
}
