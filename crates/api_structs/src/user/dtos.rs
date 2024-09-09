use nittei_domain::{Metadata, User, ID};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserDTO {
    pub id: ID,
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
