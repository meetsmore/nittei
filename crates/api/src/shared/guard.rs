use nittei_domain::ID;

use crate::error::NitteiError;

pub struct Guard {}

impl Guard {
    pub fn against_malformed_id(val: String) -> Result<ID, NitteiError> {
        val.parse()
            .map_err(|e| NitteiError::BadClientData(format!("{}", e)))
    }
}
