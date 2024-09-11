use std::convert::{TryFrom, TryInto};

use nittei_domain::{Service, ServiceWithUsers, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    FromRow,
    PgPool,
};
use tracing::{error, instrument};

use super::IServiceRepo;
use crate::repos::{service_user::ServiceUserRaw, shared::query_structs::MetadataFindQuery};

#[derive(Debug)]
pub struct PostgresServiceRepo {
    pool: PgPool,
}

impl PostgresServiceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct ServiceRaw {
    service_uid: Uuid,
    account_uid: Uuid,
    multi_person: Value,
    metadata: Value,
}

#[derive(Debug, FromRow)]
struct ServiceWithUsersRaw {
    service_uid: Uuid,
    account_uid: Uuid,
    users: Option<Value>,
    multi_person: Value,
    metadata: Value,
}

impl TryFrom<ServiceRaw> for Service {
    type Error = anyhow::Error;
    fn try_from(e: ServiceRaw) -> anyhow::Result<Self> {
        Ok(Self {
            id: e.service_uid.into(),
            account_id: e.account_uid.into(),
            multi_person: serde_json::from_value(e.multi_person)?,
            metadata: serde_json::from_value(e.metadata)?,
        })
    }
}

impl TryFrom<ServiceWithUsersRaw> for ServiceWithUsers {
    type Error = anyhow::Error;
    fn try_from(e: ServiceWithUsersRaw) -> anyhow::Result<Self> {
        let users: Vec<ServiceUserRaw> = match e.users {
            Some(json) => serde_json::from_value(json).unwrap_or_default(),
            None => Vec::new(),
        };
        Ok(Self {
            id: e.service_uid.into(),
            account_id: e.account_uid.into(),
            users: users.into_iter().map(|u| u.into()).collect(),
            multi_person: serde_json::from_value(e.multi_person)?,
            metadata: serde_json::from_value(e.metadata)?,
        })
    }
}

#[async_trait::async_trait]
impl IServiceRepo for PostgresServiceRepo {
    #[instrument]
    async fn insert(&self, service: &Service) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO services(service_uid, account_uid, multi_person, metadata)
            VALUES($1, $2, $3, $4)
            "#,
            service.id.as_ref(),
            service.account_id.as_ref(),
            Json(&service.multi_person) as _,
            Json(&service.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Unable to insert service: {:?}. DB returned error: {:?}",
                service, e
            );
        })?;

        Ok(())
    }

    #[instrument]
    async fn save(&self, service: &Service) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE services SET
                multi_person = $2,
                metadata = $3
            WHERE service_uid = $1
            "#,
            service.id.as_ref(),
            Json(&service.multi_person) as _,
            Json(&service.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Unable to save service: {:?}. DB returned error: {:?}",
                service, e
            );
        })?;

        Ok(())
    }

    #[instrument]
    async fn find(&self, service_id: &ID) -> anyhow::Result<Option<Service>> {
        sqlx::query_as!(
            ServiceRaw,
            r#"
            SELECT * FROM services AS s
            WHERE s.service_uid = $1
            "#,
            service_id.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find service with id: {:?} failed. DB returned error: {:?}",
                service_id, e
            );
        })?
        .map(|service| service.try_into())
        .transpose()
    }

    #[instrument]
    async fn find_with_users(&self, service_id: &ID) -> anyhow::Result<Option<ServiceWithUsers>> {
        sqlx::query_as!(
            ServiceWithUsersRaw,
            r#"
            SELECT s.*, jsonb_agg((su.*)) AS users FROM services AS s
            LEFT JOIN service_users AS su
            ON su.service_uid = s.service_uid
            WHERE s.service_uid = $1
            GROUP BY s.service_uid
            "#,
            service_id.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find service with id: {:?} failed. DB returned error: {:?}",
                service_id, e
            );
        })?
        .map(|service| service.try_into())
        .transpose()
    }

    #[instrument]
    async fn delete(&self, service_id: &ID) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM services AS s
            WHERE s.service_uid = $1
            "#,
            service_id.as_ref(),
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Delete service with id: {:?} failed. DB returned error: {:?}",
                service_id, e
            );
        })
        .map_err(Into::into)
        .map(|_| ())
    }

    #[instrument]
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> anyhow::Result<Vec<Service>> {
        sqlx::query_as!(
            ServiceRaw,
            r#"
            SELECT * FROM services AS s
            WHERE s.account_uid = $1 AND metadata @> $2
            LIMIT $3
            OFFSET $4
            "#,
            query.account_id.as_ref(),
            Json(&query.metadata) as _,
            query.limit as i64,
            query.skip as i64,
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find services by metadata: {:?} failed. DB returned error: {:?}",
                query, e
            );
        })?
        .into_iter()
        .map(|s| s.try_into())
        .collect()
    }
}
