use actix_web::{HttpRequest, HttpResponse, web};
use nittei_api_structs::get_event_instances::*;
use nittei_domain::{
    CalendarEvent,
    EventInstance,
    ID,
    TimeSpan,
    expand_event_and_remove_exceptions,
    generate_map_exceptions_original_start_times,
};
use nittei_infra::NitteiContext;
use tracing::error;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_event, protect_admin_route, protect_route},
        usecase::{UseCase, execute},
    },
};

pub async fn get_event_instances_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    query_params: web::Query<QueryParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_admin_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_id, &ctx).await?;

    let usecase = GetEventInstancesUseCase {
        user_id: e.user_id,
        event_id: e.id,
        timespan: query_params.0,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.event, usecase_res.instances))
        })
        .map_err(NitteiError::from)
}

pub async fn get_event_instances_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    query_params: web::Query<QueryParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetEventInstancesUseCase {
        user_id: user.id.clone(),
        event_id: path_params.event_id.clone(),
        timespan: query_params.0,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            HttpResponse::Ok().json(APIResponse::new(usecase_res.event, usecase_res.instances))
        })
        .map_err(NitteiError::from)
}

#[derive(Debug)]
pub struct GetEventInstancesUseCase {
    pub user_id: ID,
    pub event_id: ID,
    pub timespan: QueryParams,
}

#[derive(Debug)]
pub enum UseCaseError {
    InternalError,
    NotFound(String, ID),
    InvalidTimespan,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::InternalError => Self::InternalError,
            UseCaseError::InvalidTimespan => {
                Self::BadClientData("The provided start_ts and end_ts is invalid".into())
            }
            UseCaseError::NotFound(entity, event_id) => Self::NotFound(format!(
                "The {} with id: {}, was not found.",
                entity, event_id
            )),
        }
    }
}

#[derive(Debug)]
pub struct UseCaseResponse {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetEventInstancesUseCase {
    type Response = UseCaseResponse;

    type Error = UseCaseError;

    const NAME: &'static str = "GetEventInstances";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        // Get the event and its exceptions (for recurring event, if any)
        let event_and_exceptions = ctx
            .repos
            .events
            .find_by_id_and_recurring_event_id(&self.event_id)
            .await
            .map_err(|_| UseCaseError::InternalError)?;

        // Search for the main event in the list of events
        let main_event = event_and_exceptions.iter().find(|e| e.id == self.event_id);

        // If the main event is not found, return an error
        let main_event = main_event.ok_or(UseCaseError::NotFound(
            "CalendarEvent".into(),
            self.event_id.clone(),
        ))?;

        // If the user_id of the main event is different from the user_id of the user, return an error
        if self.user_id != main_event.user_id {
            return Err(UseCaseError::NotFound(
                "CalendarEvent".into(),
                self.event_id.clone(),
            ));
        }

        // Check if the timespan is valid
        let timespan = TimeSpan::new(self.timespan.start_time, self.timespan.end_time);
        if timespan.greater_than(ctx.config.event_instances_query_duration_limit) {
            return Err(UseCaseError::InvalidTimespan);
        }

        // Get the calendar of the main event
        let calendar = match ctx.repos.calendars.find(&main_event.calendar_id).await {
            Ok(Some(cal)) => cal,
            Ok(None) => {
                return Err(UseCaseError::NotFound(
                    "Calendar".into(),
                    main_event.calendar_id.clone(),
                ));
            }
            Err(_) => {
                return Err(UseCaseError::InternalError);
            }
        };

        // Generate a map of exceptions based on their original start times
        // No need to exclude the main event from the map, as it will not have an original_start_time
        // This creates a map of recurring_event_id to a list of original_start_times
        // Here, technically, we only have 1 event, so we only have 1 recurring_event_id
        let map_exceptions = generate_map_exceptions_original_start_times(&event_and_exceptions);

        // In this case, we only have 1 recurring_event_id
        // so we already get the list of exceptions (their original_start_times)
        let exceptions = map_exceptions
            .get(&main_event.id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        // Expand the event and remove the exceptions
        let instances =
            expand_event_and_remove_exceptions(&calendar, main_event, exceptions, &timespan)
                .map_err(|e| {
                    error!("Got an error while expanding an event {:?}", e);
                    UseCaseError::InternalError
                })?;

        Ok(UseCaseResponse {
            event: main_event.clone(),
            instances,
        })
    }
}
