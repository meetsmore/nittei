use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::{account_search_events::*, dtos::CalendarEventDTO};
use nittei_domain::{CalendarEventSort, DateTimeQuery, ID, IDQuery, StringQuery};
use nittei_infra::{NitteiContext, SearchEventsForAccountParams, SearchEventsParams};

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn account_search_events_controller(
    http_req: HttpRequest,
    body: actix_web_validator::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = AccountSearchEventsUseCase {
        account_id: account.id,
        user_id: body.filter.user_id,
        external_parent_id: body.filter.external_parent_id,
        start_time: body.filter.start_time,
        end_time: body.filter.end_time,
        status: body.filter.status,
        event_type: body.filter.event_type,
        updated_at: body.filter.updated_at,
        is_recurring: body.filter.is_recurring,
        metadata: body.filter.metadata,
        sort: body.sort,
        limit: body.limit,
    };

    execute(usecase, &ctx)
        .await
        .map(|events| HttpResponse::Ok().json(APIResponse::new(events.events)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct AccountSearchEventsUseCase {
    /// Account ID
    pub account_id: ID,

    /// Optional query on user ID, or list of user IDs
    pub user_id: Option<IDQuery>,

    /// Optional query on parent ID (which is a string as it's an ID from an external system)
    pub external_parent_id: Option<StringQuery>,

    /// Optional query on start time - "lower than or equal", or "great than or equal" (UTC)
    pub start_time: Option<DateTimeQuery>,

    /// Optional query on end time - "lower than or equal", or "great than or equal" (UTC)
    pub end_time: Option<DateTimeQuery>,

    /// Optional query on event type
    pub event_type: Option<StringQuery>,

    /// Optional query on event status
    pub status: Option<StringQuery>,

    /// Optional query on updated at - "lower than or equal", or "great than or equal" (UTC)
    pub updated_at: Option<DateTimeQuery>,

    /// Optional recurrence test
    pub is_recurring: Option<bool>,

    /// Optional list of metadata key-value pairs
    pub metadata: Option<serde_json::Value>,

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
    BadRequest,
    InternalError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::BadRequest => Self::BadClientData("Bad request".to_string()),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for AccountSearchEventsUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "SearchEvents";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<UseCaseResponse, UseCaseError> {
        if let Some(limit) = self.limit {
            // Note that limit is unsigned, so it can't be negative
            if limit == 0 {
                return Err(UseCaseError::BadRequest);
            }
        }

        let res = ctx
            .repos
            .events
            .search_events_for_account(SearchEventsForAccountParams {
                account_id: self.account_id.clone(),
                search_events_params: SearchEventsParams {
                    user_id: self.user_id.take(),
                    external_parent_id: self.external_parent_id.take(),
                    start_time: self.start_time.take(),
                    end_time: self.end_time.take(),
                    status: self.status.take(),
                    event_type: self.event_type.take(),
                    updated_at: self.updated_at.take(),
                    is_recurring: self.is_recurring.take(),
                    metadata: self.metadata.take(),
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
