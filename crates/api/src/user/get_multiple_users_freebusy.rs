use std::collections::HashMap;

use axum::{Extension, Json, http::HeaderMap};
use chrono::{DateTime, Utc};
use futures::{FutureExt, StreamExt, future::join_all, stream};
use nittei_api_structs::multiple_freebusy::{
    MultipleFreeBusyAPIResponse,
    MultipleFreeBusyRequestBody,
};
use nittei_domain::{
    Calendar,
    CalendarEvent,
    EventInstance,
    ID,
    TimeSpan,
    expand_all_events_and_remove_exceptions,
};
use nittei_infra::NitteiContext;
use nittei_utils::config::APP_CONFIG;
use tracing::error;

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_public_account_route,
        usecase::{UseCase, execute},
    },
};

#[utoipa::path(
    post,
    tag = "User",
    path = "/api/v1/user/freebusy",
    summary = "Get freebusy for multiple users",
    security(
        ("api_key" = [])
    ),
    request_body(
        content = MultipleFreeBusyRequestBody,
    ),
    responses(
        (status = 200, body = MultipleFreeBusyAPIResponse)
    )
)]
pub async fn get_multiple_freebusy_controller(
    headers: HeaderMap,
    Extension(ctx): Extension<NitteiContext>,
    body: Json<MultipleFreeBusyRequestBody>,
) -> Result<Json<MultipleFreeBusyAPIResponse>, NitteiError> {
    let account = protect_public_account_route(&headers, &ctx).await?;

    let usecase = GetMultipleFreeBusyUseCase {
        account_id: account.id,
        user_ids: body.user_ids.clone(),
        start_time: body.start_time,
        end_time: body.end_time,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| Json(MultipleFreeBusyAPIResponse(usecase_res.0)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetMultipleFreeBusyUseCase {
    pub account_id: ID,
    pub user_ids: Vec<ID>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct GetMultipleFreeBusyResponse(pub HashMap<ID, Vec<EventInstance>>);

#[derive(Debug)]
pub enum UseCaseError {
    InternalError,
    InvalidTimespan,
    UserNotFound(ID),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::InvalidTimespan => {
                Self::BadClientData("The provided start_ts and end_ts are invalid".into())
            }
            UseCaseError::UserNotFound(user_id) => {
                Self::NotFound(format!("A user with id: {user_id}, was not found."))
            }
        }
    }
}

#[async_trait::async_trait]
impl UseCase for GetMultipleFreeBusyUseCase {
    type Response = GetMultipleFreeBusyResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetMultipleFreebusy";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        self.ensure_users_belong_to_account(ctx).await?;

        let timespan = TimeSpan::new(self.start_time, self.end_time);
        if timespan.greater_than(APP_CONFIG.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        // TODO: Implement this
        // if self.user_ids.is_empty() {
        //     return Err(UseCaseError::InvalidTimespan);
        // }

        let calendars = self
            .get_calendars_from_user_ids(ctx)
            .await
            .map_err(|_| UseCaseError::InternalError)?;

        let busy_event_instances = self
            .get_event_instances_from_calendars(timespan, ctx, calendars)
            .await?;

        Ok(GetMultipleFreeBusyResponse(busy_event_instances))
    }
}

impl GetMultipleFreeBusyUseCase {
    async fn ensure_users_belong_to_account(&self, ctx: &NitteiContext) -> Result<(), UseCaseError> {
        for user_id in &self.user_ids {
            let user = ctx
                .repos
                .users
                .find_by_account_id(user_id, &self.account_id)
                .await
                .map_err(|_| UseCaseError::InternalError)?;
            if user.is_none() {
                return Err(UseCaseError::UserNotFound(user_id.clone()));
            }
        }
        Ok(())
    }

    async fn get_calendars_from_user_ids(
        &self,
        ctx: &NitteiContext,
    ) -> anyhow::Result<Vec<Calendar>> {
        let calendars: Vec<anyhow::Result<Vec<Calendar>>> = join_all(
            self.user_ids
                .iter()
                .map(|user_id| ctx.repos.calendars.find_by_user(user_id)),
        )
        .await
        .into_iter()
        .collect();

        let mut all_calendars = Vec::new();
        for res in calendars {
            match res {
                Ok(cals) => all_calendars.extend(cals),
                Err(_) => return Err(anyhow::anyhow!("Internal error")),
            }
        }

        Ok(all_calendars)
    }

    async fn get_event_instances_from_calendars(
        &self,
        timespan: TimeSpan,
        ctx: &NitteiContext,
        calendars: Vec<Calendar>,
    ) -> Result<HashMap<ID, Vec<EventInstance>>, UseCaseError> {
        // For quick lookup by calendar id
        let calendars_lookup = calendars
            .iter()
            .map(|cal| (cal.id.to_string(), cal))
            .collect::<HashMap<_, _>>();

        // End result
        let mut events_per_user: HashMap<ID, Vec<EventInstance>> = HashMap::new();

        // Group calendars by user once so each user is queried with the same semantics
        // as the single-user freebusy endpoint.
        let mut calendar_ids_by_user: HashMap<ID, Vec<ID>> = HashMap::new();
        for calendar in &calendars {
            calendar_ids_by_user
                .entry(calendar.user_id.clone())
                .or_default()
                .push(calendar.id.clone());
        }

        // Fetch all events for all users (using calendar groups)
        // This is not executed yet (lazy)
        let all_events_futures = calendar_ids_by_user.into_iter().map(|(user_id, calendar_ids)| {
            let timespan = timespan.clone();

            async move {
                let events = ctx
                    .repos
                    .events
                    .find_busy_events_and_recurring_events_for_calendars(
                        &calendar_ids,
                        timespan,
                        false,
                    )
                    .await
                    .map_err(|_| UseCaseError::InternalError)?;
                Ok((user_id, events))
                    as Result<(ID, Vec<CalendarEvent>), UseCaseError>
            }
            .boxed()
        });

        // Fetch events in chunks of 5
        let mut futures_stream = stream::iter(all_events_futures).chunks(5);

        // Fetch events in parallel (actual execution)
        while let Some(futures) = futures_stream.next().await {
            let events_res = join_all(futures).await;

            for event_result in events_res {
                match event_result {
                    Ok((user_id, events)) => {
                        let timespan = timespan.clone();
                        let expanded_events = expand_all_events_and_remove_exceptions(
                            &calendars_lookup,
                            &events,
                            timespan.clone(),
                        )
                        .map_err(|e| {
                            error!("Got an error when expanding events {:?}", e);
                            UseCaseError::InternalError
                        })?;
                        events_per_user
                            .entry(user_id)
                            .or_default()
                            .extend(expanded_events);
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        // Sort the events by start time
        for (_, events) in events_per_user.iter_mut() {
            events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        }

        Ok(events_per_user)
    }
}

#[cfg(test)]
mod test {
    use nittei_domain::{Account, Calendar, CalendarEvent, Entity, RRuleOptions, User};
    use nittei_infra::setup_context;

    use super::*;

    #[tokio::test]
    async fn multiple_freebusy_works() {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id(), &user.account_id, None, None);
        ctx.repos.calendars.insert(&calendar).await.unwrap();
        let second_calendar = Calendar::new(&user.id(), &user.account_id, None, None);
        ctx.repos.calendars.insert(&second_calendar).await.unwrap();
        let one_hour = 1000 * 60 * 60;
        let mut e1 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            status: nittei_domain::CalendarEventStatus::Confirmed,
            duration: one_hour,
            end_time: DateTime::<Utc>::MAX_UTC,
            ..Default::default()
        };
        let e1rr = RRuleOptions {
            count: Some(100),
            ..Default::default()
        };
        match e1.set_recurrence(e1rr) {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        };

        let mut e2 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            status: nittei_domain::CalendarEventStatus::Confirmed,
            duration: one_hour,
            end_time: DateTime::<Utc>::MAX_UTC,
            start_time: DateTime::from_timestamp_millis(one_hour * 4).unwrap(),
            ..Default::default()
        };
        let e2rr = RRuleOptions {
            count: Some(100),
            ..Default::default()
        };
        match e2.set_recurrence(e2rr) {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        };

        let mut e3 = CalendarEvent {
            calendar_id: calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            status: nittei_domain::CalendarEventStatus::Confirmed,
            duration: one_hour,
            end_time: DateTime::from_timestamp_millis(one_hour).unwrap(),
            ..Default::default()
        };
        let e3rr = RRuleOptions {
            count: Some(100),
            interval: 2,
            ..Default::default()
        };
        match e3.set_recurrence(e3rr) {
            Ok(_) => {}
            Err(e) => {
                panic!("{e:?}");
            }
        };
        let e4 = CalendarEvent {
            calendar_id: second_calendar.id.clone(),
            user_id: user.id.clone(),
            account_id: user.account_id.clone(),
            busy: true,
            status: nittei_domain::CalendarEventStatus::Confirmed,
            start_time: DateTime::from_timestamp_millis(90000000).unwrap(),
            end_time: DateTime::from_timestamp_millis(93600000).unwrap(),
            duration: one_hour,
            ..Default::default()
        };

        ctx.repos.events.insert(&e1).await.unwrap();
        ctx.repos.events.insert(&e2).await.unwrap();
        ctx.repos.events.insert(&e3).await.unwrap();
        ctx.repos.events.insert(&e4).await.unwrap();

        let mut usecase = GetMultipleFreeBusyUseCase {
            account_id: account.id.clone(),
            user_ids: vec![user.id().clone()],
            start_time: DateTime::from_timestamp_millis(86400000).unwrap(),
            end_time: DateTime::from_timestamp_millis(172800000).unwrap(),
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
        let map_instances = res.unwrap().0;
        assert_eq!(map_instances.len(), 1);
        assert!(map_instances.contains_key(&user.id()));
        let instances = map_instances.get(&user.id()).unwrap();
        assert_eq!(instances.len(), 5);
        assert_eq!(
            instances[0],
            EventInstance {
                busy: true,
                start_time: DateTime::from_timestamp_millis(86400000).unwrap(),
                end_time: DateTime::from_timestamp_millis(90000000).unwrap(),
            }
        );
        assert_eq!(
            instances[1],
            EventInstance {
                busy: true,
                start_time: DateTime::from_timestamp_millis(90000000).unwrap(),
                end_time: DateTime::from_timestamp_millis(93600000).unwrap(),
            }
        );
        assert_eq!(
            instances[2],
            EventInstance {
                busy: true,
                start_time: DateTime::from_timestamp_millis(100800000).unwrap(),
                end_time: DateTime::from_timestamp_millis(104400000).unwrap(),
            }
        );
        assert_eq!(
            instances[3],
            EventInstance {
                busy: true,
                start_time: DateTime::from_timestamp_millis(172800000).unwrap(),
                end_time: DateTime::from_timestamp_millis(176400000).unwrap(),
            }
        );
        assert_eq!(
            instances[4],
            EventInstance {
                busy: true,
                start_time: DateTime::from_timestamp_millis(172800000).unwrap(),
                end_time: DateTime::from_timestamp_millis(176400000).unwrap(),
            }
        );
    }
}
