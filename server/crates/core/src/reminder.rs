use crate::shared::entity::Entity;

#[derive(Debug, Clone)]
pub struct Reminder {
    pub id: String,
    pub event_id: String,
    pub account_id: String,
    pub remind_at: i64,
    pub priority: i64,
}

impl Entity for Reminder {
    fn id(&self) -> String {
        self.id.clone()
    }
}

#[derive(Debug, Clone)]
pub struct EventRemindersExpansionJob {
    pub id: String,
    pub event_id: String,
    pub timestamp: i64,
}

impl Entity for EventRemindersExpansionJob {
    fn id(&self) -> String {
        self.id.clone()
    }
}