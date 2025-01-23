use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::{account_search_events::*, dtos::CalendarEventDTO};
use nittei_domain::{DateTimeQuery, IDQuery, StringQuery, ID};
use nittei_infra::{NitteiContext, SearchEventsForAccountParams, SearchEventsParams};

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn account_search_events_controller(
    http_req: HttpRequest,
    body: actix_web_validator::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = AccountSearchEventsUseCase {
        account_id: account.id,
        parent_id: body.parent_id,
        group_id: body.group_id,
        start_time: body.start_time,
        end_time: body.end_time,
        status: body.status,
        event_type: body.event_type,
        updated_at: body.updated_at,
        metadata: body.metadata,
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

    /// Optional query on parent ID (which is a string as it's an ID from an external system)
    pub parent_id: Option<StringQuery>,

    /// Optional query on the group ID
    pub group_id: Option<IDQuery>,

    /// Optional query on start time - "lower than or equal", or "great than or equal" (UTC)
    pub start_time: Option<DateTimeQuery>,

    /// Optional query on end time - "lower than or equal", or "great than or equal" (UTC)
    pub end_time: Option<DateTimeQuery>,

    /// Optional query on event type
    pub event_type: Option<StringQuery>,

    /// Optional query on event status
    pub status: Option<StringQuery>,

    /// Optioanl query on updated at - "lower than or equal", or "great than or equal" (UTC)
    pub updated_at: Option<DateTimeQuery>,

    /// Optional list of metadata key-value pairs
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct UseCaseResponse {
    pub events: Vec<CalendarEventDTO>,
}

#[derive(Debug)]
pub enum UseCaseError {
    InternalError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for AccountSearchEventsUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "SearchEvents";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<UseCaseResponse, UseCaseError> {
        let res = ctx
            .repos
            .events
            .search_events_for_account(SearchEventsForAccountParams {
                account_id: self.account_id.clone(),
                search_events_params: SearchEventsParams {
                    parent_id: self.parent_id.take(),
                    group_id: self.group_id.take(),
                    start_time: self.start_time.take(),
                    end_time: self.end_time.take(),
                    status: self.status.take(),
                    event_type: self.event_type.take(),
                    updated_at: self.updated_at.take(),
                    metadata: self.metadata.take(),
                },
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
