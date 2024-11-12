use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

/// Query parameters for searching on a date time
#[derive(Deserialize, Serialize, TS, Debug, Validate, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "DateTimeQuery")]
pub struct DateTimeQuery {
    /// Optional "greater than or equal" query (UTC)
    #[ts(type = "Date")]
    pub gte: Option<DateTime<Utc>>,

    /// Optional "less than or equal" query (UTC)
    #[ts(type = "Date")]
    pub lte: Option<DateTime<Utc>>,
}
