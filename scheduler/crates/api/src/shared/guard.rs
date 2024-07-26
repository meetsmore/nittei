use nettu_scheduler_domain::ID;

use crate::error::NettuError;

pub struct Guard {}

impl Guard {
    pub fn against_malformed_id(val: String) -> Result<ID, NettuError> {
        val.parse()
            .map_err(|e| NettuError::BadClientData(format!("{}", e)))
    }
}
