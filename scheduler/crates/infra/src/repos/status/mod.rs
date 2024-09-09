pub use postgres::PostgresStatusRepo;

mod postgres;

#[async_trait::async_trait]
pub trait IStatusRepo: Send + Sync {
    async fn check_connection(&self) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {

    use crate::setup_context;

    #[tokio::test]
    async fn check_connection() {
        let ctx = setup_context().await.unwrap();

        // Should not panic
        ctx.repos.status.check_connection().await.unwrap();
    }
}
