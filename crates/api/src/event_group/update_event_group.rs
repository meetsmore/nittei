use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::update_event_group::*;
use nittei_domain::{event_group::EventGroup, User, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_event, account_can_modify_user, protect_account_route},
        usecase::{execute, UseCase},
    },
};

pub async fn update_event_group_admin_controller(
    http_req: HttpRequest,
    body: web::Json<RequestBody>,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let e = account_can_modify_event(&account, &path_params.event_group_id, &ctx).await?;
    let user = account_can_modify_user(&account, &e.user_id, &ctx).await?;

    let body = body.0;
    let usecase = UpdateEventGroupUseCase {
        user,
        event_group_id: e.id,
        parent_id: body.parent_id,
        external_id: body.external_id,
    };

    execute(usecase, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(APIResponse::new(event)))
        .map_err(NitteiError::from)
}

#[derive(Debug, Default)]
pub struct UpdateEventGroupUseCase {
    pub user: User,
    pub event_group_id: ID,

    pub parent_id: Option<String>,
    pub external_id: Option<String>,
}

#[derive(Debug)]
pub enum UseCaseError {
    NotFound(String, ID),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(entity, event_id) => Self::NotFound(format!(
                "The {} with id: {}, was not found.",
                entity, event_id
            )),
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for UpdateEventGroupUseCase {
    type Response = EventGroup;

    type Error = UseCaseError;

    const NAME: &'static str = "UpdateGroupEvent";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let UpdateEventGroupUseCase {
            user,
            event_group_id,
            parent_id,
            external_id,
        } = self;

        let mut g = match ctx.repos.event_groups.find(event_group_id).await {
            Ok(Some(event_group)) if event_group.user_id == user.id => event_group,
            Ok(_) => {
                return Err(UseCaseError::NotFound(
                    "Calendar Event".into(),
                    event_group_id.clone(),
                ))
            }
            Err(e) => {
                tracing::error!("Failed to get one event {:?}", e);
                return Err(UseCaseError::StorageError);
            }
        };

        if parent_id.is_some() {
            g.parent_id.clone_from(parent_id);
        }

        if external_id.is_some() {
            g.external_id.clone_from(external_id);
        }

        // e.updated = ctx.sys.get_timestamp_millis();

        ctx.repos
            .event_groups
            .save(&g)
            .await
            .map(|_| g.clone())
            .map_err(|_| UseCaseError::StorageError)
    }
}

// #[cfg(test)]
// mod test {
//     use nittei_infra::setup_context;

//     use super::*;

//     #[actix_web::main]
//     #[test]
//     async fn update_nonexisting_event() {
//         let mut usecase = UpdateEventGroupUseCase {
//             start_time: Some(DateTime::from_timestamp_millis(500).unwrap()),
//             duration: Some(800),
//             busy: Some(false),
//             ..Default::default()
//         };
//         let ctx = setup_context().await.unwrap();
//         let res = usecase.execute(&ctx).await;
//         assert!(res.is_err());
//     }
// }
