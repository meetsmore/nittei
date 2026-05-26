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

    /// Liveness probe — checks that the process is running.
    pub async fn check_liveness(&self) -> APIResponse<get_service_health::APIResponse> {
        self.base
            .get("healthz/live".into(), None, StatusCode::OK)
            .await
    }

    /// Readiness probe — checks that the service can handle traffic (DB reachable).
    pub async fn check_readiness(&self) -> APIResponse<get_service_health::APIResponse> {
        self.base
            .get("healthz/ready".into(), None, StatusCode::OK)
            .await
    }
}
