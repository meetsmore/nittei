mod postgres;

use nettu_scheduler_domain::{Schedule, ID};
pub use postgres::PostgresScheduleRepo;

use crate::MetadataFindQuery;

#[async_trait::async_trait]
pub trait IScheduleRepo: Send + Sync {
    async fn insert(&self, schedule: &Schedule) -> anyhow::Result<()>;
    async fn save(&self, schedule: &Schedule) -> anyhow::Result<()>;
    async fn find(&self, schedule_id: &ID) -> anyhow::Result<Option<Schedule>>;
    async fn find_many(&self, schedule_ids: &[ID]) -> anyhow::Result<Vec<Schedule>>;
    async fn find_by_user(&self, user_id: &ID) -> anyhow::Result<Vec<Schedule>>;
    async fn delete(&self, schedule_id: &ID) -> anyhow::Result<()>;
    async fn find_by_metadata(&self, query: MetadataFindQuery) -> anyhow::Result<Vec<Schedule>>;
}

#[cfg(test)]
mod tests {
    use chrono_tz::US::Pacific;
    use nettu_scheduler_domain::{Account, Entity, Schedule, User};

    use crate::setup_context;

    #[tokio::test]
    async fn create_and_delete() {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos
            .accounts
            .insert(&account)
            .await
            .expect("To insert account");
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.expect("To insert user");

        let schedule = Schedule::new(user.id.clone(), account.id.clone(), &Pacific);

        // Insert
        assert!(ctx.repos.schedules.insert(&schedule).await.is_ok());

        // Different find methods
        let res = ctx
            .repos
            .schedules
            .find(&schedule.id)
            .await
            .unwrap()
            .unwrap();
        assert!(res.eq(&schedule));
        let res = ctx
            .repos
            .schedules
            .find_many(&[schedule.id.clone()])
            .await
            .unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].eq(&schedule));

        // Delete
        let res = ctx.repos.schedules.delete(&schedule.id).await;
        assert!(res.is_ok());

        // Find
        assert!(ctx
            .repos
            .schedules
            .find(&schedule.id)
            .await
            .unwrap()
            .is_none());

        // Insert again
        assert!(ctx.repos.schedules.insert(&schedule).await.is_ok());

        // Delete by user
        ctx.repos.users.delete(&user.id).await.expect("Delete user");
        assert!(ctx
            .repos
            .schedules
            .find(&schedule.id)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn update() {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos
            .accounts
            .insert(&account)
            .await
            .expect("To insert account");
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.expect("To insert user");

        let mut schedule = Schedule::new(user.id.clone(), account.id.clone(), &Pacific);

        // Insert
        assert!(ctx.repos.schedules.insert(&schedule).await.is_ok());

        assert_eq!(schedule.rules.len(), 7);
        schedule.rules = Vec::new();

        // Save
        assert!(ctx.repos.schedules.save(&schedule).await.is_ok());

        // Find
        assert!(ctx
            .repos
            .schedules
            .find(&schedule.id)
            .await
            .unwrap()
            .unwrap()
            .rules
            .is_empty());
    }
}
