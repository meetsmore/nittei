use sqlx::PgPool;
use tracing::instrument;

use super::IStatusRepo;

/// The Postgres status repository
#[derive(Debug)]
pub struct PostgresStatusRepo {
    pool: PgPool,
}

impl PostgresStatusRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IStatusRepo for PostgresStatusRepo {
    #[instrument]
    async fn check_connection(&self) -> anyhow::Result<()> {
        // Send a simple query to check the connection
        sqlx::query("SELECT 1 AS health")
            .execute(&self.pool)
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to check connection: {:?}", e);
            })?;

        Ok(())
    }
}
