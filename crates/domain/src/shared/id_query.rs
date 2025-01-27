use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ID;

/// Query parameters for searching on an ID (or list of IDs)
#[derive(Deserialize, Serialize, TS, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "IDQuery")]
pub enum IDQuery {
    /// ID (equality test)
    Eq(ID),

    /// ID (inequality test)
    Ne(ID),

    /// Bool (existence test)
    Exists(bool),

    /// List of IDs (equality test)
    In(Vec<ID>),
}
