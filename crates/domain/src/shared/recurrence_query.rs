use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Query parameters for searching on a recurrence
#[derive(Deserialize, Serialize, TS, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export, rename = "RecurrenceQuery", rename_all = "camelCase")]
pub enum RecurrenceQuery {
    /// Bool (existence test)
    Exists(bool),

    /// Exists and is recurring at a specific date
    #[ts(type = "Date")]
    ExistsAndRecurringAt(DateTime<Utc>),
}
