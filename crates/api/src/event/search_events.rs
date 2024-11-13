use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::{dtos::CalendarEventDTO, search_events::*};
use nittei_domain::{DateTimeQuery, IDQuery, ID};
use nittei_infra::{NitteiContext, SearchEventsParams};

use crate::{
    error::NitteiError,
    shared::{
        auth::protect_account_route,
        usecase::{execute, UseCase},
    },
};

pub async fn search_events_controller(
    http_req: HttpRequest,
    body: actix_web_validator::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;

    let body = body.0;
    let usecase = SearchEventsUseCase {
        account_id: account.id,
        user_id: body.user_id,
        calendar_ids: body.calendar_ids,
        parent_id: body.parent_id,
        start_time: body.start_time,
        end_time: body.end_time,
        status: body.status,
        updated_at: body.updated_at,
        metadata: body.metadata,
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

    /// Optional query on parent ID
    pub parent_id: Option<IDQuery>,

    /// Optional query on start time - "lower than or equal", or "great than or equal" (UTC)
    pub start_time: Option<DateTimeQuery>,

    /// Optional query on end time - "lower than or equal", or "great than or equal" (UTC)
    pub end_time: Option<DateTimeQuery>,

    /// Optional list of event status
    pub status: Option<Vec<String>>,

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
            .search_events(SearchEventsParams {
                user_id: self.user_id.clone(),
                calendar_ids: self.calendar_ids.clone(),
                parent_id: self.parent_id.clone(),
                start_time: self.start_time.clone(),
                end_time: self.end_time.clone(),
                status: self.status.clone(),
                updated_at: self.updated_at.clone(),
                metadata: self.metadata.clone(),
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
