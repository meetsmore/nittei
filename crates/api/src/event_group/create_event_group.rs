use actix_web::{web, HttpRequest, HttpResponse};
use nittei_api_structs::create_event_group::*;
use nittei_domain::{event_group::EventGroup, User, ID};
use nittei_infra::NitteiContext;

use crate::{
    error::NitteiError,
    shared::{
        auth::{account_can_modify_user, protect_account_route},
        usecase::{execute, UseCase},
    },
};

pub async fn create_event_group_admin_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    body: actix_web_validator::Json<RequestBody>,
    ctx: web::Data<NitteiContext>,
) -> Result<HttpResponse, NitteiError> {
    let account = protect_account_route(&http_req, &ctx).await?;
    let user = account_can_modify_user(&account, &path_params.user_id, &ctx).await?;

    let body = body.0;
    let usecase = CreateEventGroupUseCase {
        parent_id: body.parent_id,
        external_id: body.external_id,
        user,
        calendar_id: body.calendar_id,
    };

    execute(usecase, &ctx)
        .await
        .map(|group| HttpResponse::Created().json(APIResponse::new(group)))
        .map_err(NitteiError::from)
}

#[derive(Debug, Default)]
pub struct CreateEventGroupUseCase {
    pub calendar_id: ID,
    pub user: User,
    pub parent_id: Option<String>,
    pub external_id: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum UseCaseError {
    NotFound(ID),
    StorageError,
}

impl From<UseCaseError> for NitteiError {
    fn from(e: UseCaseError) -> Self {
        match e {
            UseCaseError::NotFound(calendar_id) => Self::NotFound(format!(
                "The calendar with id: {}, was not found.",
                calendar_id
            )),
            UseCaseError::StorageError => Self::InternalError,
        }
    }
}

impl From<anyhow::Error> for UseCaseError {
    fn from(_: anyhow::Error) -> Self {
        UseCaseError::StorageError
    }
}

#[async_trait::async_trait(?Send)]
impl UseCase for CreateEventGroupUseCase {
    type Response = EventGroup;

    type Error = UseCaseError;

    const NAME: &'static str = "CreateEvent";

    async fn execute(&mut self, ctx: &NitteiContext) -> Result<Self::Response, Self::Error> {
        let calendar = ctx
            .repos
            .calendars
            .find(&self.calendar_id)
            .await
            .map_err(|_| UseCaseError::StorageError)?;
        let calendar = match calendar {
            Some(calendar) if calendar.user_id == self.user.id => calendar,
            _ => return Err(UseCaseError::NotFound(self.calendar_id.clone())),
        };

        let g = EventGroup {
            id: Default::default(),
            parent_id: self.parent_id.clone(),
            external_id: self.external_id.clone(),
            calendar_id: calendar.id.clone(),
            user_id: self.user.id.clone(),
            account_id: self.user.account_id.clone(),
        };

        ctx.repos.event_groups.insert(&g).await?;

        Ok(g)
    }
}

#[cfg(test)]
mod test {
    use nittei_domain::{Account, Calendar, User};
    use nittei_infra::setup_context;

    use super::*;

    struct TestContext {
        ctx: NitteiContext,
        calendar: Calendar,
        user: User,
    }

    async fn setup() -> TestContext {
        let ctx = setup_context().await.unwrap();
        let account = Account::default();
        ctx.repos.accounts.insert(&account).await.unwrap();
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let calendar = Calendar::new(&user.id, &account.id, None, None);
        ctx.repos.calendars.insert(&calendar).await.unwrap();

        TestContext {
            user,
            calendar,
            ctx,
        }
    }

    #[actix_web::main]
    #[test]
    async fn creates_event_group() {
        let TestContext {
            ctx,
            calendar,
            user,
        } = setup().await;

        let mut usecase = CreateEventGroupUseCase {
            calendar_id: calendar.id.clone(),
            user,
            ..Default::default()
        };

        let res = usecase.execute(&ctx).await;

        assert!(res.is_ok());
    }

    #[actix_web::main]
    #[test]
    async fn rejects_invalid_calendar_id() {
        let TestContext {
            ctx,
            calendar: _,
            user,
        } = setup().await;

        let mut usecase = CreateEventGroupUseCase {
            user,
            ..Default::default()
        };

        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            UseCaseError::NotFound(usecase.calendar_id)
        );
    }
}
