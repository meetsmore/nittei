use std::sync::Arc;

use nittei_api_structs::*;
use reqwest::StatusCode;

use crate::{APIResponse, BaseClient};

#[derive(Clone)]
pub struct StatusClient {
    base: Arc<BaseClient>,
}

impl StatusClient {
    pub(crate) fn new(base: Arc<BaseClient>) -> Self {
        Self { base }
    }

    pub async fn check_health(&self) -> APIResponse<get_service_health::APIResponse> {
        self.base
            .get("healthcheck".into(), None, StatusCode::OK)
            .await
    }
}
