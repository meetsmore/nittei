use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::{dtos::CalendarEventDTO, search_events::*};
use nittei_domain::{CalendarEventSort, DateTimeQuery, ID, IDQuery, StringQuery};
use nittei_infra::{NitteiContext, SearchEventsForUserParams, SearchEventsParams};

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_admin_route,
        usecase::{UseCase, execute},
    },
};

pub async fn search_events_controller(
    http_req: HttpRequest,
    body: actix_web_validator::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = SearchEventsUseCase {
        account_id: account.id,
        user_id: body.filter.user_id,
        calendar_ids: body.filter.calendar_ids,
        external_id: body.filter.external_id,
        external_parent_id: body.filter.external_parent_id,
        start_time: body.filter.start_time,
        end_time: body.filter.end_time,
        event_type: body.filter.event_type,
        status: body.filter.status,
        updated_at: body.filter.updated_at,
        original_start_time: body.filter.original_start_time,
        is_recurring: body.filter.is_recurring,
        metadata: body.filter.metadata,
        sort: body.sort,
        limit: body.limit.or(Some(200)), // Default limit to 200
    };

    execute(usecase, &ctx)
        .await
        .map(|events| HttpResponse::Ok().json(APIResponse::new(events.events)))
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct SearchEventsUseCase {
    /// Account ID
    pub account_id: ID,

    /// User ID
    pub user_id: ID,

    /// Optional list of calendar UUIDs
    /// If not provided, all calendars will be used
    pub calendar_ids: Option<Vec<ID>>,

    /// Optional query on ID (which is a string as it's an ID from an external system)
    pub external_id: Option<StringQuery>,

    /// Optional query on external parent ID (which is a string as it's an ID from an external system)
    pub external_parent_id: Option<StringQuery>,

    /// Optional query on start time - "lower than or equal", or "great than or equal" (UTC)
    pub start_time: Option<DateTimeQuery>,

    /// Optional query on end time - "lower than or equal", or "great than or equal" (UTC)
    pub end_time: Option<DateTimeQuery>,

    /// Optional query on event type
    pub event_type: Option<StringQuery>,

    /// Optional query on status
    pub status: Option<StringQuery>,

    /// Optioanl query on updated at - "lower than or equal", or "great than or equal" (UTC)
    pub updated_at: Option<DateTimeQuery>,

    /// Optional list of metadata key-value pairs
    pub metadata: Option<serde_json::Value>,

    /// Optional query on original start time - "lower than or equal", or "great than or equal" (UTC)
    pub original_start_time: Option<DateTimeQuery>,

    /// Optional filter on the recurrence (existence)
    pub is_recurring: Option<bool>,

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
    InternalError,
    BadRequest(String),
    NotFound(String, String),
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::BadRequest(msg) => Self::BadClientData(msg),
            UseCaseError::NotFound(entity, event_id) => Self::NotFound(format!(
                "The {} with id: {}, was not found.",
                entity, event_id
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for SearchEventsUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "SearchEvents";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<UseCaseResponse, UseCaseError> {
        if let Some(calendar_ids) = &self.calendar_ids {
            if calendar_ids.is_empty() {
                return Err(UseCaseError::BadRequest(
                    "calendar_ids cannot be empty".into(),
                ));
            }

            let calendars = ctx
                .repos
                .calendars
                .find_multiple(calendar_ids.iter().collect())
                .await
                .map_err(|_| UseCaseError::InternalError)?;

            // Check that all calendars exist and belong to the same account
            if calendars.is_empty()
                || calendars.len() != calendar_ids.len()
                || !calendars
                    .iter()
                    .all(|cal| cal.account_id == self.account_id)
            {
                return Err(UseCaseError::NotFound(
                    "Calendars not found".to_string(),
                    calendar_ids
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<String>>()
                        .join(","),
                ));
            }
        }

        let res = ctx
            .repos
            .events
            .search_events_for_user(SearchEventsForUserParams {
                user_id: self.user_id.clone(),
                calendar_ids: self.calendar_ids.take(),
                search_events_params: SearchEventsParams {
                    // Force user_id to be the same as the one in the search
                    user_id: Some(IDQuery::Eq(self.user_id.clone())),
                    external_id: self.external_id.take(),
                    external_parent_id: self.external_parent_id.take(),
                    start_time: self.start_time.take(),
                    end_time: self.end_time.take(),
                    event_type: self.event_type.take(),
                    status: self.status.take(),
                    updated_at: self.updated_at.take(),
                    original_start_time: self.original_start_time.take(),
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
