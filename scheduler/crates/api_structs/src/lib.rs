mod account;
mod calendar;
mod event;
mod schedule;
mod service;
mod status;
mod user;
pub mod dtos {
    pub use crate::{
        account::dtos::*,
        calendar::dtos::*,
        event::dtos::*,
        schedule::dtos::*,
        service::dtos::*,
        user::dtos::*,
    };
}
pub use crate::{
    account::api::*,
    calendar::api::*,
    event::api::*,
    schedule::api::*,
    service::api::*,
    status::api::*,
    user::api::*,
};
