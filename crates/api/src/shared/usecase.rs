use std::fmt::Debug;

use futures::future::join_all;
use nittei_infra::NitteiContext;
use tracing::{debug, warn};

use super::auth::{Permission, Policy};
use crate::error::NitteiError;

/// Subscriber is a side effect to a `UseCase`
///
/// It is going to act upon the response of the execution
/// of the `UseCase` if the execution was a success.
#[async_trait::async_trait]
pub trait Subscriber<U: UseCase>: Send + Sync {
    async fn notify(&self, e: &U::Response, ctx: &NitteiContext);
}

#[async_trait::async_trait]
pub trait UseCase: Debug + Send {
    type Response: Debug + Send;
    type Error: Send;

    /// UseCase name identifier
    const NAME: &'static str;

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error>;

    fn subscribers() -> Vec<Box<dyn Subscriber<Self>>> {
        Default::default()
    }
}

/// Restrict what `Permission`s are needed for a `User`
/// to be able to execute the `UseCase`
pub trait PermissionBoundary: UseCase {
    fn permissions(&self) -> Vec<Permission>;
}

#[derive(Debug)]
pub enum UseCaseErrorContainer<T: Debug> {
    Unauthorized(String),
    UseCase(T),
}

impl<T> From<UseCaseErrorContainer<T>> for NitteiError
where
    NitteiError: From<T>,
    T: Debug,
{
    fn from(e: UseCaseErrorContainer<T>) -> Self {
        match e {
            UseCaseErrorContainer::Unauthorized(e) => NitteiError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => e.into(),
        }
    }
}

#[tracing::instrument(name = "UseCase executed by User", skip(usecase, policy, ctx), fields(usecase = %U::NAME))]
pub async fn execute_with_policy<U>(
    usecase: U,
    policy: &Policy,
    ctx: &NitteiContext,
) -> Result<U::Response, UseCaseErrorContainer<U::Error>>
where
    U: PermissionBoundary,
    U::Error: Debug,
{
    let required_permissions = usecase.permissions();
    if !policy.authorize(&required_permissions) {
        let err = format!(
            "Client is not permitted to perform some or all of these actions: {:?}",
            required_permissions
        );
        warn!("{}", err);
        return Err(UseCaseErrorContainer::Unauthorized(err));
    }

    _execute(usecase, ctx)
        .await
        .map_err(UseCaseErrorContainer::UseCase)
}

#[tracing::instrument(name = "UseCase executed by Account", skip(usecase, ctx), fields(usecase = %U::NAME))]
pub async fn execute<U>(usecase: U, ctx: &NitteiContext) -> Result<U::Response, U::Error>
where
    U: UseCase + Send,
    U::Error: Debug + Send,
{
    _execute(usecase, ctx).await
}

async fn _execute<U>(mut usecase: U, ctx: &NitteiContext) -> Result<U::Response, U::Error>
where
    U: UseCase + Send,
    U::Error: Debug + Send,
{
    debug!("{:?}", usecase);
    let res = usecase.execute(ctx).await;

    match &res {
        Ok(res) => {
            let subscribers = U::subscribers();
            let mut subscriber_promises = Vec::with_capacity(subscribers.len());
            for subscriber in &subscribers {
                subscriber_promises.push(subscriber.notify(res, ctx));
            }
            join_all(subscriber_promises).await;
        }
        Err(e) => {
            // This is debug because the error is usually already logged deeper in the stack
            debug!("Error: {:?}", e);
        }
    }

    res
}
