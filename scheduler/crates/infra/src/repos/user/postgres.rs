use std::convert::{TryFrom, TryInto};

use nettu_scheduler_domain::{User, ID};
use serde_json::Value;
use sqlx::{
    types::{Json, Uuid},
    FromRow,
    PgPool,
};
use tracing::{error, instrument};

use super::IUserRepo;
use crate::repos::shared::query_structs::MetadataFindQuery;

#[derive(Debug)]
pub struct PostgresUserRepo {
    pool: PgPool,
}

impl PostgresUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct UserRaw {
    user_uid: Uuid,
    account_uid: Uuid,
    metadata: Value,
}

impl TryFrom<UserRaw> for User {
    type Error = anyhow::Error;
    fn try_from(e: UserRaw) -> anyhow::Result<Self> {
        Ok(Self {
            id: e.user_uid.into(),
            account_id: e.account_uid.into(),
            metadata: serde_json::from_value(e.metadata)?,
        })
    }
}

#[async_trait::async_trait]
impl IUserRepo for PostgresUserRepo {
    #[instrument]
    async fn insert(&self, user: &User) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO users(user_uid, account_uid, metadata)
            VALUES($1, $2, $3)
            "#,
            user.id.as_ref(),
            user.account_id.as_ref(),
            Json(&user.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Unable to insert user: {:?}. DB returned error: {:?}",
                user, e
            );
        })?;

        Ok(())
    }

    #[instrument]
    async fn save(&self, user: &User) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET account_uid = $2,
            metadata = $3
            WHERE user_uid = $1
            "#,
            user.id.as_ref(),
            user.account_id.as_ref(),
            Json(&user.metadata) as _,
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Unable to save user: {:?}. DB returned error: {:?}",
                user, e
            );
        })?;
        Ok(())
    }

    #[instrument]
    async fn delete(&self, user_id: &ID) -> anyhow::Result<Option<User>> {
        let res = sqlx::query_as!(
            UserRaw,
            r#"
            DELETE FROM users AS u
            WHERE u.user_uid = $1
            RETURNING *
            "#,
            user_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Delete user with id: {} failed. DB returned error: {:?}",
                user_id, e
            );
        })?;

        res.map(|user| user.try_into()).transpose()
    }

    #[instrument]
    async fn find(&self, user_id: &ID) -> anyhow::Result<Option<User>> {
        let res = sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.user_uid = $1
            "#,
            user_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find user with user_id: {} failed. DB returned error: {:?}",
                user_id, e
            );
        })?;

        res.map(|user| user.try_into()).transpose()
    }

    #[instrument]
    async fn find_many(&self, user_ids: &[ID]) -> anyhow::Result<Vec<User>> {
        let user_ids = user_ids.iter().map(|id| *id.as_ref()).collect::<Vec<_>>();

        let users: Vec<UserRaw> = sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.user_uid = ANY($1)
            "#,
            &user_ids
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find users with user_ids: {:?} failed. DB returned error: {:?}",
                user_ids, e
            );
        })?;

        users.into_iter().map(|u| u.try_into()).collect()
    }

    #[instrument]
    async fn find_by_account_id(
        &self,
        user_id: &ID,
        account_id: &ID,
    ) -> anyhow::Result<Option<User>> {
        let res = sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.user_uid = $1 AND
            u.account_uid = $2
            "#,
            user_id.as_ref(),
            account_id.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find user with user_id: {} failed. DB returned error: {:?}",
                user_id, e
            );
        })?;

        res.map(|user| user.try_into()).transpose()
    }

    #[instrument]
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> anyhow::Result<Vec<User>> {
        let users: Vec<UserRaw> = sqlx::query_as!(
            UserRaw,
            r#"
            SELECT * FROM users AS u
            WHERE u.account_uid = $1 AND metadata @> $2
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
                "Find users by metadata: {:?} failed. DB returned error: {:?}",
                query, e
            );
        })?;

        users.into_iter().map(|u| u.try_into()).collect()
    }
}
