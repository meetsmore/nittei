use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Entity, ID};

/// Metadata - key-value pairs for storing additional information
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Metadata {
    #[serde(flatten)]
    pub inner: HashMap<String, serde_json::Value>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn new_kv(key: String, value: String) -> Self {
        let mut inner = HashMap::new();
        inner.insert(key, serde_json::Value::String(value));
        Self::from(inner)
    }
}

impl From<HashMap<String, serde_json::Value>> for Metadata {
    fn from(inner: HashMap<String, serde_json::Value>) -> Self {
        Self { inner }
    }
}

pub trait Meta<T: PartialEq>: Entity<T> {
    fn metadata(&self) -> &Metadata;
    /// Retrieves the account_id associated with this entity, which
    /// is useful to know when querying on the metadata
    fn account_id(&self) -> &ID;
}
