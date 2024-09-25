use nittei_domain::{Metadata, User, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserDTO {
    pub id: ID,
    #[ts(type = "Record<string, string>")]
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
