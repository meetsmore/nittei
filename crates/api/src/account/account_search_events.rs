use axum::{Extension, Json};
use axum_valid::Valid;
use nittei_api_structs::{account_search_events::*, dtos::CalendarEventDTO};
use nittei_domain::{
    Account,
    CalendarEventSort,
    DateTimeQuery,
    ID,
    IDQuery,
    RecurrenceQuery,
    StringQuery,
};
use nittei_infra::{NitteiContext, SearchEventsForAccountParams, SearchEventsParams};
use nittei_utils::config::APP_CONFIG;

use crate::{
    error::NitteiError,
    shared::usecase::{UseCase, execute},
};

#[utoipa::path(
    get,
    tag = "Account",
    path = "/api/v1/account/search-events",
    summary = "Search events inside an account",
    security(
        ("api_key" = [])
    ),
    request_body(
        content = AccountSearchEventsRequestBody,
    ),
    responses(
        (status = 200, description = "The found events", body = SearchEventsAPIResponse)
    )
)]
/// Search events inside an account
pub async fn account_search_events_controller(
    Extension(account): Extension<Account>,
    Extension(ctx): Extension<NitteiContext>,
    body: Valid<Json<AccountSearchEventsRequestBody>>,
) -> Result<Json<SearchEventsAPIResponse>, NitteiError> {
    let mut body = body.0;
    let usecase = AccountSearchEventsUseCase {
        account_uid: account.id,
        event_uid: body.filter.event_uid.take(),
        user_uid: body.filter.user_id.take(),
        external_id: body.filter.external_id.take(),
        external_parent_id: body.filter.external_parent_id.take(),
        start_time: body.filter.start_time.take(),
        end_time: body.filter.end_time.take(),
        status: body.filter.status.take(),
        event_type: body.filter.event_type.take(),
        recurring_event_uid: body.filter.recurring_event_uid.take(),
        original_start_time: body.filter.original_start_time.take(),
        recurrence: body.filter.recurrence.take(),
        metadata: body.filter.metadata.take(),
        created_at: body.filter.created_at.take(),
        updated_at: body.filter.updated_at.take(),
        sort: body.sort.take(),
        limit: body.limit.or(Some(1000)), // Default limit to 1000
    };

    execute(usecase, &ctx)
        .await
        .map(|events| Json(SearchEventsAPIResponse::new(events.events)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct AccountSearchEventsUseCase {
    /// Account UUID
    pub account_uid: ID,

    /// Optional query on event UUID, or list of event UUIDs
    pub event_uid: Option<IDQuery>,

    /// Optional query on user UUID, or list of user UUIDs
    pub user_uid: Option<IDQuery>,

    /// Optional query on external ID (which is a string as it's an ID from an external system)
    pub external_id: Option<StringQuery>,

    /// Optional query on external parent ID (which is a string as it's an ID from an external system)
    pub external_parent_id: Option<StringQuery>,

    /// Optional query on start time - "lower than or equal", or "great than or equal" (UTC)
    pub start_time: Option<DateTimeQuery>,

    /// Optional query on end time - "lower than or equal", or "great than or equal" (UTC)
    pub end_time: Option<DateTimeQuery>,

    /// Optional query on event type
    pub event_type: Option<StringQuery>,

    /// Optional query on event status
    pub status: Option<StringQuery>,

    /// Optional query on the recurring event UID
    pub recurring_event_uid: Option<IDQuery>,

    /// Optional query on original start time - "lower than or equal", or "great than or equal" (UTC)
    pub original_start_time: Option<DateTimeQuery>,

    /// Optional recurrence query
    /// This allows to filter on the existence or not of a recurrence, or the existence of a recurrence at a specific date
    pub recurrence: Option<RecurrenceQuery>,

    /// Optional list of metadata key-value pairs
    pub metadata: Option<serde_json::Value>,

    /// Optional query on created at - "lower than or equal", or "great than or equal" (UTC)
    pub created_at: Option<DateTimeQuery>,

    /// Optional query on updated at - "lower than or equal", or "great than or equal" (UTC)
    pub updated_at: Option<DateTimeQuery>,

    /// Optional sort
    pub sort: Option<CalendarEventSort>,

    /// Optional limit
    pub limit: Option<u16>,
}

#[derive(Debug)]
pub struct UseCaseResponse {
    pub events: Vec<CalendarEventDTO>,
}

#[derive(Debug)]
pub enum UseCaseError {
    BadRequest(String),
    InternalError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::BadRequest(msg) => Self::BadClientData(msg),
        }
    }
}

#[async_trait::async_trait]
impl UseCase for AccountSearchEventsUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "SearchEvents";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<UseCaseResponse, UseCaseError> {
        if let Some(limit) = self.limit {
            // Note that limit is unsigned, so it can't be negative
            // Limit nb of events to be returned
            if limit == 0 || limit > APP_CONFIG.max_events_returned_by_search {
                return Err(UseCaseError::BadRequest(format!(
                    "Limit is invalid: it should be positive and under {}",
                    APP_CONFIG.max_events_returned_by_search
                )));
            }
        }

        let res = ctx
            .repos
            .events
            .search_events_for_account(SearchEventsForAccountParams {
                account_id: self.account_uid.clone(),
                search_events_params: SearchEventsParams {
                    event_uid: self.event_uid.take(),
                    user_uid: self.user_uid.take(),
                    external_id: self.external_id.take(),
                    external_parent_id: self.external_parent_id.take(),
                    start_time: self.start_time.take(),
                    end_time: self.end_time.take(),
                    status: self.status.take(),
                    event_type: self.event_type.take(),
                    recurring_event_uid: self.recurring_event_uid.take(),
                    original_start_time: self.original_start_time.take(),
                    recurrence: self.recurrence.take(),
                    metadata: self.metadata.take(),
                    created_at: self.created_at.take(),
                    updated_at: self.updated_at.take(),
                },
                sort: self.sort.take(),
                limit: self.limit.take(),
            })
            .await;

        match res {
            Ok(events) => Ok(UseCaseResponse {
                events: events.into_iter().map(CalendarEventDTO::new).collect(),
            }),
            Err(_) => Err(UseCaseError::InternalError),
        }
    }
}
