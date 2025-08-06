#![allow(clippy::unwrap_used)]
// Allow unwrap used because we are using lazy_static, and the only way to handle errors is to unwrap

use lazy_static::lazy_static;
use prometheus::{IntGauge, Registry};

lazy_static! {
    pub static ref INFRA_REGISTRY: Registry = Registry::new();
    pub static ref DB_CONNECTION_POOL_TOTAL: IntGauge = IntGauge::new(
        "db_connection_pool_total",
        "Total number of connections in the pool"
    )
    .unwrap();
    pub static ref DB_CONNECTION_POOL_IDLE: IntGauge = IntGauge::new(
        "db_connection_pool_idle",
        "Number of idle connections in the pool"
    )
    .unwrap();
    pub static ref DB_CONNECTION_POOL_BUSY: IntGauge = IntGauge::new(
        "db_connection_pool_busy",
        "Number of busy connections in the pool"
    )
    .unwrap();
}

pub fn register_metrics() -> anyhow::Result<()> {
    INFRA_REGISTRY.register(Box::new(DB_CONNECTION_POOL_TOTAL.clone()))?;
    INFRA_REGISTRY.register(Box::new(DB_CONNECTION_POOL_IDLE.clone()))?;
    INFRA_REGISTRY.register(Box::new(DB_CONNECTION_POOL_BUSY.clone()))?;

    Ok(())
}

pub fn update_connection_pool_metrics(total: i64, idle: i64, busy: i64) {
    DB_CONNECTION_POOL_TOTAL.set(total);
    DB_CONNECTION_POOL_IDLE.set(idle);
    DB_CONNECTION_POOL_BUSY.set(busy);
}
