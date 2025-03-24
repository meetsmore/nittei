use std::time::Duration;

use actix_web::rt::time::{Instant, interval, sleep_until};
use nittei_api_structs::send_event_reminders::AccountRemindersDTO;
use nittei_infra::NitteiContext;
use reqwest::Client;
use tracing::{debug, error};

use crate::{
    event::{
        get_upcoming_reminders::GetUpcomingRemindersUseCase,
        sync_event_reminders::{SyncEventRemindersTrigger, SyncEventRemindersUseCase},
    },
    shared::usecase::execute,
};

/// Get the delay in seconds to the next minute
pub fn get_start_delay(now_ts: usize, secs_before_min: usize) -> usize {
    let secs_to_next_minute = 60 - (now_ts / 1000) % 60;
    if secs_to_next_minute > secs_before_min {
        secs_to_next_minute - secs_before_min
    } else {
        secs_to_next_minute + (60 - secs_before_min)
    }
}

/// Start the job scheduler for generating reminders
pub fn start_reminder_generation_job(ctx: NitteiContext) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30 * 60));
        loop {
            interval.tick().await;

            let usecase = SyncEventRemindersUseCase {
                request: SyncEventRemindersTrigger::JobScheduler,
            };
            let _ = execute(usecase, &ctx).await;
        }
    });
}

/// Start the job scheduler for sending reminders
pub fn start_send_reminders_job(ctx: NitteiContext) {
    tokio::spawn(async move {
        let now = ctx.sys.get_timestamp_millis();
        let secs_to_next_run = get_start_delay(now as usize, 0);
        let start = Instant::now() + Duration::from_secs(secs_to_next_run as u64);

        sleep_until(start).await;
        let mut minutely_interval = interval(Duration::from_secs(60));
        loop {
            minutely_interval.tick().await;
            let context = ctx.clone();
            tokio::spawn(send_reminders(context));
        }
    });
}

/// Send reminders to clients
async fn send_reminders(context: NitteiContext) {
    let client = Client::new();

    let usecase = GetUpcomingRemindersUseCase {
        reminders_interval: 1000 * 60,
    };
    let account_reminders = match execute(usecase, &context).await {
        Ok(res) => res,
        Err(_) => return,
    };

    let send_instant = account_reminders.1;
    sleep_until(send_instant).await;
    debug!(
        "Reminders to send at {} : {:?}",
        context.sys.get_timestamp_millis(),
        account_reminders
    );

    for (acc, reminders) in account_reminders.0 {
        match acc.settings.webhook {
            None => continue,
            Some(webhook) => {
                if let Err(e) = client
                    .post(webhook.url)
                    .header("nittei-scheduler-webhook-key", webhook.key)
                    .json(&AccountRemindersDTO::new(reminders))
                    .send()
                    .await
                {
                    error!("Error informing client of reminders: {:?}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_delay_works() {
        assert_eq!(get_start_delay(50 * 1000, 5), 5);
        assert_eq!(get_start_delay(50 * 1000, 10), 60);
        assert_eq!(get_start_delay(50 * 1000, 15), 55);
        assert_eq!(get_start_delay(60 * 1000, 60), 60);
        assert_eq!(get_start_delay(60 * 1000, 10), 50);
        assert_eq!(get_start_delay(59 * 1000, 0), 1);
        assert_eq!(get_start_delay(59 * 1000, 1), 60);
    }
}
