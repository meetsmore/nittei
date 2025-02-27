use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
    time::Duration,
};

use moka::future::Cache;
use nittei_domain::{Account, ID, PEMKey};
use serde_json::Value;
use sqlx::{
    FromRow,
    PgPool,
    types::{Json, Uuid},
};
use tracing::{error, instrument};

use super::IAccountRepo;

#[derive(Debug)]
pub struct PostgresAccountRepo {
    pool: PgPool,
    cache: Arc<Cache<String, Account>>,
}

impl PostgresAccountRepo {
    pub fn new(pool: PgPool) -> Self {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(300)) // 5 minutes
            .build();

        Self {
            pool,
            cache: Arc::new(cache),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct AccountRaw {
    account_uid: Uuid,
    secret_api_key: String,
    public_jwt_key: Option<String>,
    settings: Value,
}

impl TryFrom<AccountRaw> for Account {
    type Error = anyhow::Error;

    fn try_from(e: AccountRaw) -> anyhow::Result<Self> {
        let pem_key = if let Some(public_jwt_key) = e.public_jwt_key {
            Some(PEMKey::new(public_jwt_key)?)
        } else {
            None
        };
        Ok(Self {
            id: e.account_uid.into(),
            secret_api_key: e.secret_api_key,
            public_jwt_key: pem_key,
            settings: serde_json::from_value(e.settings)?,
        })
    }
}

#[async_trait::async_trait]
impl IAccountRepo for PostgresAccountRepo {
    #[instrument]
    async fn insert(&self, account: &Account) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO accounts(account_uid, secret_api_key, public_jwt_key, settings)
            VALUES($1, $2, $3, $4)
            "#,
            account.id.as_ref(),
            account.secret_api_key,
            account.public_jwt_key.clone().map(|key| key.inner()),
            Json(&account.settings) as _
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Unable to insert account: {:?}. DB returned error: {:?}",
                account, e
            );
        })?;
        Ok(())
    }

    #[instrument]
    async fn save(&self, account: &Account) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE accounts
            SET secret_api_key = $2,
            public_jwt_key = $3,
            settings = $4
            WHERE account_uid = $1
            "#,
            account.id.as_ref(),
            account.secret_api_key,
            account.public_jwt_key.clone().map(|key| key.inner()),
            Json(&account.settings) as _
        )
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Unable to save account: {:?}. DB returned error: {:?}",
                account, e
            );
        })?;
        Ok(())
    }

    #[instrument]
    async fn find(&self, account_id: &ID) -> anyhow::Result<Option<Account>> {
        sqlx::query_as!(
            AccountRaw,
            r#"
            SELECT * FROM accounts
            WHERE account_uid = $1
            "#,
            account_id.as_ref(),
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find account with id: {:?} failed. DB returned error: {:?}",
                account_id, e
            );
        })?
        .map(|res| res.try_into())
        .transpose()
    }

    #[instrument]
    async fn find_many(&self, accounts_ids: &[ID]) -> anyhow::Result<Vec<Account>> {
        let ids = accounts_ids
            .iter()
            .map(|id| *id.as_ref())
            .collect::<Vec<_>>();
        let accounts_raw: Vec<AccountRaw> = sqlx::query_as!(
            AccountRaw,
            "
            SELECT * FROM accounts
            WHERE account_uid = ANY($1)
            ",
            &ids
        )
        .fetch_all(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find accounts with ids: {:?} failed. DB returned error: {:?}",
                accounts_ids, e
            );
        })?;

        // Use map and try_into for each account and collect the results
        accounts_raw
            .into_iter()
            .map(|acc| acc.try_into()) // Apply try_into to each AccountRaw
            .collect::<Result<Vec<Account>, _>>() // Collect into Result<Vec<Account>, _>
    }

    #[instrument]
    async fn delete(&self, account_id: &ID) -> anyhow::Result<Option<Account>> {
        sqlx::query_as!(
            AccountRaw,
            "
            DELETE FROM accounts
            WHERE account_uid = $1
            RETURNING *
            ",
            account_id.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Delete account with id: {:?} failed. DB returned error: {:?}",
                account_id, e
            );
        })?
        .map(|res| res.try_into())
        .transpose()
    }

    #[instrument]
    async fn find_by_apikey(&self, api_key: &str) -> anyhow::Result<Option<Account>> {
        if let Some(account) = self.cache.get(api_key).await {
            return Ok(Some(account));
        }

        let result: Option<Account> = sqlx::query_as!(
            AccountRaw,
            "
            SELECT * FROM accounts
            WHERE secret_api_key = $1
            ",
            api_key
        )
        .fetch_optional(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                "Find account with api_key: {:?} failed. DB returned error: {:?}",
                api_key, e
            );
        })?
        .map(|res| res.try_into())
        .transpose()?;

        if let Some(ref account) = result {
            self.cache
                .insert(api_key.to_string(), account.clone())
                .await;
        }

        Ok(result)
    }
}
