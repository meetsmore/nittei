mod create_schedule;
mod delete_schedule;
mod get_schedule;
mod get_schedules_by_meta;
mod update_schedule;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use create_schedule::{create_schedule_admin_controller, create_schedule_controller};
use delete_schedule::{delete_schedule_admin_controller, delete_schedule_controller};
use get_schedule::{get_schedule_admin_controller, get_schedule_controller};
use get_schedules_by_meta::get_schedules_by_meta_controller;
use update_schedule::{update_schedule_admin_controller, update_schedule_controller};

pub fn configure_routes(cfg: &mut Router) {
    cfg.route("/schedule", post(create_schedule_controller));
    cfg.route(
        "/user/{user_id}/schedule",
        post(create_schedule_admin_controller),
    );

    cfg.route("/schedule/meta", get(get_schedules_by_meta_controller));

    cfg.route("/schedule/{schedule_id}", get(get_schedule_controller));
    cfg.route(
        "/user/schedule/{schedule_id}",
        get(get_schedule_admin_controller),
    );

    cfg.route(
        "/schedule/{schedule_id}",
        delete(delete_schedule_controller),
    );
    cfg.route(
        "/user/schedule/{schedule_id}",
        delete(delete_schedule_admin_controller),
    );

    cfg.route("/schedule/{schedule_id}", put(update_schedule_controller));
    cfg.route(
        "/user/schedule/{schedule_id}",
        put(update_schedule_admin_controller),
    );
}
