use nittei_infra::run_migration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_migration().await
}
