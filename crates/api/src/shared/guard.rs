use nittei_domain::ID;

use crate::error::NitteiError;

/// Empty struct used to namespace guard functions
pub struct Guard {}

impl Guard {
    /// Guard function to check if the provided ID is malformed
    pub fn against_malformed_id(val: String) -> Result<ID, NitteiError> {
        val.parse()
            .map_err(|e| NitteiError::BadClientData(format!("{e}")))
    }
}
