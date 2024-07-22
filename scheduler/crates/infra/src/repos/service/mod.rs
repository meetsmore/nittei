mod postgres;

use super::shared::query_structs::MetadataFindQuery;
use nettu_scheduler_domain::{Service, ServiceWithUsers, ID};
pub use postgres::PostgresServiceRepo;

#[async_trait::async_trait]
pub trait IServiceRepo: Send + Sync {
    async fn insert(&self, service: &Service) -> anyhow::Result<()>;
    async fn save(&self, service: &Service) -> anyhow::Result<()>;
    async fn find(&self, service_id: &ID) -> Option<Service>;
    async fn find_with_users(&self, service_id: &ID) -> Option<ServiceWithUsers>;
    async fn delete(&self, service_id: &ID) -> anyhow::Result<()>;
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<Service>;
}

#[cfg(test)]
mod tests {
    use crate::setup_context;
    use nettu_scheduler_domain::{Account, Metadata, Service, ServiceResource, TimePlan, User};

    #[tokio::test]
    async fn create_and_delete() {
        let ctx = setup_context().await;
        let account = Account::default();
        ctx.repos
            .accounts
            .insert(&account)
            .await
            .expect("To insert account");
        let service = Service::new(account.id.clone());

        // Insert
        assert!(ctx.repos.services.insert(&service).await.is_ok());

        // Get by id
        let mut service = ctx
            .repos
            .services
            .find(&service.id)
            .await
            .expect("To get service");

        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();

        let timeplan = TimePlan::Empty;
        let resource = ServiceResource::new(user.id.clone(), service.id.clone(), timeplan);
        assert!(ctx.repos.service_users.insert(&resource).await.is_ok());

        let mut metadata = Metadata::new();
        metadata.inner.insert("foo".to_string(), "bar".to_string());
        service.metadata = metadata;
        ctx.repos
            .services
            .save(&service)
            .await
            .expect("To save service");

        let service = ctx
            .repos
            .services
            .find_with_users(&service.id)
            .await
            .expect("To get service");
        assert_eq!(
            *service.metadata.inner.get("foo").unwrap(),
            "bar".to_string()
        );
        assert_eq!(service.users.len(), 1);

        ctx.repos
            .users
            .delete(&user.id)
            .await
            .expect("To delete user");

        let service = ctx
            .repos
            .services
            .find_with_users(&service.id)
            .await
            .expect("To get service");
        assert!(service.users.is_empty());

        ctx.repos
            .services
            .delete(&service.id)
            .await
            .expect("To delete service");

        assert!(ctx.repos.services.find(&service.id).await.is_none());
    }
}
