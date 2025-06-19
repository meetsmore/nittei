mod policy;
mod route_guards;

pub use policy::{Permission, Policy};
pub use route_guards::{
    NITTEI_X_API_KEY_HEADER,
    account_can_modify_calendar,
    account_can_modify_event,
    account_can_modify_schedule,
    account_can_modify_user,
    protect_admin_route,
    protect_public_account_route,
    protect_route,
};
