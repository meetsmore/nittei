use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::ID;

/// Query parameters for searching on an ID (or list of IDs)
#[derive(Deserialize, Serialize, TS, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export, rename = "IDQuery", rename_all = "camelCase")]
pub enum IDQuery {
    /// ID (equality test)
    Eq(ID),

    /// ID (inequality test)
    Ne(ID),

    /// Bool (existence test)
    Exists(bool),

    /// List of IDs (equality test)
    In(Vec<ID>),

    /// List of IDs (inequality test)
    Nin(Vec<ID>),

    /// Greater than the ID
    Gt(ID),

    /// Greater than or equal to the ID
    Gte(ID),

    /// Less than the ID
    Lt(ID),

    /// Less than or equal to the ID
    Lte(ID),
}
