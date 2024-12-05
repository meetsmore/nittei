mod postgres;

use nittei_domain::{event_group::EventGroup, ID};
pub use postgres::PostgresEventGroupRepo;

#[async_trait::async_trait]
pub trait IEventGroupRepo: Send + Sync {
    async fn insert(&self, e: &EventGroup) -> anyhow::Result<()>;
    async fn save(&self, e: &EventGroup) -> anyhow::Result<()>;
    async fn find(&self, group_id: &ID) -> anyhow::Result<Option<EventGroup>>;
    async fn get_by_external_id(&self, external_id: &str) -> anyhow::Result<Option<EventGroup>>;
    async fn delete(&self, group_id: &ID) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use nittei_domain::{event_group::EventGroup, Account, Calendar, User, ID};

    use crate::{setup_context, NitteiContext};

    fn generate_default_event_group(account_id: &ID, calendar_id: &ID, user_id: &ID) -> EventGroup {
        EventGroup {
            account_id: account_id.clone(),
            calendar_id: calendar_id.clone(),
            user_id: user_id.clone(),
            ..Default::default()
        }
    }

    struct TestContext {
        ctx: NitteiContext,
        account: Account,
        calendar: Calendar,
        user: User,
    }

    async fn setup() -> TestContext {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id, None, None);
        ctx.repos.calendars.insert(&calendar).await.unwrap();

        TestContext {
            account,
            calendar,
            user,
            ctx,
        }
    }

    #[tokio::test]
    async fn create_and_delete() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let event_group = generate_default_event_group(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.event_groups.insert(&event_group).await.is_ok());

        // Get
        let get_event_group_res = ctx
            .repos
            .event_groups
            .find(&event_group.id)
            .await
            .unwrap()
            .unwrap();
        assert!(get_event_group_res.eq(&event_group));

        // Delete
        let delete_res = ctx.repos.event_groups.delete(&event_group.id).await;
        assert!(delete_res.is_ok());

        // Find
        assert!(ctx
            .repos
            .event_groups
            .find(&event_group.id)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn update() {
        let TestContext {
            ctx,
            account,
            calendar,
            user,
        } = setup().await;
        let mut event_group = generate_default_event_group(&account.id, &calendar.id, &user.id);

        // Insert
        assert!(ctx.repos.event_groups.insert(&event_group).await.is_ok());

        event_group.external_id = Some("test".to_string());

        // Save
        assert!(ctx.repos.event_groups.save(&event_group).await.is_ok());

        // Find
        assert!(ctx
            .repos
            .event_groups
            .find(&event_group.id)
            .await
            .unwrap()
            .expect("To be event_group")
            .eq(&event_group));
    }
}
