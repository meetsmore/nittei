use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Query parameters for searching on a date time
#[derive(Deserialize, Serialize, TS, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "DateTimeQuery")]
pub enum DateTimeQuery {
    /// "equal" query (UTC)
    #[ts(type = "Date")]
    Eq(DateTime<Utc>),

    /// "greater than or equal" query (UTC)
    #[ts(type = "Date")]
    Gte(DateTime<Utc>),

    /// "less than or equal" query (UTC)
    #[ts(type = "Date")]
    Lte(DateTime<Utc>),

    /// "greater than" query (UTC)
    /// This is exclusive of the value
    #[ts(type = "Date")]
    Gt(DateTime<Utc>),

    /// "less than" query (UTC)
    /// This is exclusive of the value
    #[ts(type = "Date")]
    Lt(DateTime<Utc>),
}
