mod account;
mod base;
mod calendar;
mod event;
mod schedule;
mod service;
mod shared;
mod status;
mod user;

use std::sync::Arc;

use account::AccountClient;
pub(crate) use base::BaseClient;
pub use base::{APIError, APIErrorVariant, APIResponse};
use calendar::CalendarClient;
pub use calendar::{
    CreateCalendarInput,
    GetCalendarEventsInput,
    GetGoogleCalendars,
    GetOutlookCalendars,
    StopCalendarSyncInput,
    SyncCalendarInput,
    UpdateCalendarInput,
};
use event::CalendarEventClient;
pub use event::{CreateEventInput, GetEventsInstancesInput, UpdateEventInput};
// Domain
pub use nittei_api_structs::dtos::{
    AccountDTO as Account,
    AccountSettingsDTO as AccountSettings,
    AccountWebhookSettingsDTO as AccountWebhookSettings,
    CalendarDTO as Calendar,
    CalendarEventDTO as CalendarEvent,
    CalendarSettingsDTO as CalendarSettings,
    EventWithInstancesDTO as EventWithIInstances,
    ScheduleDTO as Schedule,
    ServiceResourceDTO as ServiceResource,
    ServiceWithUsersDTO as Service,
    UserDTO as User,
};
pub use nittei_api_structs::{
    dtos::*,
    send_event_reminders::AccountRemindersDTO as AccountReminders,
};
pub use nittei_domain::{
    providers::{google::*, outlook::*},
    scheduling::RoundRobinAlgorithm,
    BusyCalendarProvider,
    CalendarEventReminder,
    IntegrationProvider,
    Metadata,
    Month,
    RRuleFrequency,
    RRuleOptions,
    ScheduleRule,
    ServiceMultiPersonOptions,
    SyncedCalendar,
    TimePlan,
    Tz,
    WeekDayRecurrence,
    Weekday,
    ID,
};
use schedule::ScheduleClient;
pub use schedule::{CreateScheduleInput, UpdateScheduleInput};
use service::ServiceClient;
pub use service::{
    AddBusyCalendar,
    AddServiceUserInput,
    CreateBookingIntendInput,
    CreateServiceInput,
    GetServiceBookingSlotsInput,
    RemoveBookingIntendInput,
    RemoveBusyCalendar,
    RemoveServiceUserInput,
    UpdateServiceInput,
    UpdateServiceUserInput,
};
pub use shared::{KVMetadata, MetadataFindInput};
use status::StatusClient;
use user::UserClient;
pub use user::{
    CreateUserInput,
    GetUserFreeBusyInput,
    MultipleFreeBusyAPIResponse,
    MultipleFreeBusyRequestBody,
    OAuthInput,
    RemoveUserIntegrationInput,
    UpdateUserInput,
};

/// nittei Server SDK
///
/// The SDK contains methods for interacting with the nittei server
/// API.
#[derive(Clone)]
pub struct NitteiSDK {
    pub account: AccountClient,
    pub calendar: CalendarClient,
    pub event: CalendarEventClient,
    pub schedule: ScheduleClient,
    pub service: ServiceClient,
    pub status: StatusClient,
    pub user: UserClient,
}

impl NitteiSDK {
    pub fn new<T: Into<String>>(address: String, api_key: T) -> Self {
        let mut base = BaseClient::new(address);
        base.set_api_key(api_key.into());
        let base = Arc::new(base);
        let account = AccountClient::new(base.clone());
        let calendar = CalendarClient::new(base.clone());
        let event = CalendarEventClient::new(base.clone());
        let schedule = ScheduleClient::new(base.clone());
        let service = ServiceClient::new(base.clone());
        let status = StatusClient::new(base.clone());
        let user = UserClient::new(base);

        Self {
            account,
            calendar,
            event,
            schedule,
            service,
            status,
            user,
        }
    }
}
