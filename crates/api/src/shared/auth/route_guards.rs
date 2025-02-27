use axum::http::HeaderMap;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use nittei_domain::{Account, Calendar, CalendarEvent, Schedule, User, ID};
use nittei_infra::NitteiContext;
use serde::{Deserialize, Serialize};
use tracing::log::warn;

use crate::{
    error::NitteiError,
    shared::{auth::Policy, Guard},
};

/// JWT Claims generated by the Identity Server and describes
/// what `Policy` the `User` has and for how long.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claims {
    /// Expiration time (as UTC timestamp)
    exp: usize,
    /// Issued at (as UTC timestamp)
    iat: usize,
    /// Subject (whom token refers tok)
    nittei_user_id: ID,
    /// The `Policy` that describes what `UseCase`s this `User` can perform
    scheduler_policy: Option<Policy>,
}

/// Parses the `Authorization` header and extracts the token
fn parse_authtoken_header(token_header_value: &str) -> String {
    if token_header_value.len() < 6 || token_header_value[..6].to_lowercase() != "bearer" {
        String::new()
    } else {
        token_header_value.trim()[6..].trim().to_string()
    }
}

/// Authenticates the user by checking the `Authorization` header
/// and decodes the token to find out which `User` is making the request
/// and what `Policy` it has
pub async fn auth_user_req(
    headers: &HeaderMap,
    account: &Account,
    ctx: &NitteiContext,
) -> anyhow::Result<Option<(User, Policy)>> {
    let token = headers.get("authorization");
    let token = match token {
        Some(token) => {
            let token = match token.to_str() {
                Ok(token) => parse_authtoken_header(token),
                Err(_) => return Ok(None),
            };
            match decode_token(account, &token) {
                // In addition to checking that the request comes with a valid jwt we also
                // have to check that the user_id actually belongs to the given `Account` that
                // signed the token
                Ok(claims) => ctx
                    .repos
                    .users
                    .find_by_account_id(&claims.nittei_user_id, &account.id)
                    .await
                    .map_err(|_| NitteiError::InternalError)?
                    .map(|user| (user, claims.scheduler_policy.unwrap_or_default())),
                Err(e) => {
                    warn!("Decode token error: {:?}", e);
                    None
                }
            }
        }
        None => None,
    };
    Ok(token)
}

/// Finds out which `Account` the client is associated with.
pub async fn get_client_account(
    headers: &HeaderMap,
    ctx: &NitteiContext,
) -> anyhow::Result<Option<Account>> {
    match get_nittei_account_header(headers) {
        Some(Ok(account_id)) => ctx.repos.accounts.find(&account_id).await,
        _ => Ok(None),
    }
}

/// Parses the `nittei-account` header and returns the `ID` of the `Account`
pub fn get_nittei_account_header(headers: &HeaderMap) -> Option<Result<ID, NitteiError>> {
    if let Some(account_id) = headers.get("nittei-account") {
        let err = NitteiError::UnidentifiableClient(format!(
            "Malformed nittei account header provided: {:?}",
            account_id
        ));

        // Validate that is is string
        let res = match account_id.to_str() {
            Ok(account_id) => Guard::against_malformed_id(account_id.to_string()),
            Err(_) => Err(err),
        };

        Some(res)
    } else {
        None
    }
}

/// Decodes the JWT token by checking if the signature matches the public
/// key provided by the `Account`
fn decode_token(account: &Account, token: &str) -> anyhow::Result<Claims> {
    let public_key = match &account.public_jwt_key {
        Some(val) => val,
        None => return Err(anyhow::Error::msg("Account does not support user tokens")),
    };
    let decoding_key = DecodingKey::from_rsa_pem(public_key.as_bytes())?;
    let claims = decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::RS256))?.claims;

    Ok(claims)
}

/// Protects routes that can be accessed by authenticated `User`s
///
/// This function will check if the request has a valid `Authorization` header
/// and if the token is valid and signed by the `Account`'s public key
pub async fn protect_route(
    headers: &HeaderMap,
    ctx: &NitteiContext,
) -> Result<(User, Policy), NitteiError> {
    let account = get_client_account(headers, ctx)
        .await
        .map_err(|_| NitteiError::InternalError)?
        .ok_or_else(|| {
            NitteiError::UnidentifiableClient(
                "Could not find out which account the client belongs to".into(),
            )
        })?;
    let res = auth_user_req(headers, &account, ctx)
        .await
        .map_err(|_| NitteiError::InternalError)?;

    match res {
        Some(user_and_policy) => Ok(user_and_policy),
        None => Err(NitteiError::Unauthorized(
            "Unable to find user from the given credentials".into(),
        )),
    }
}

/// Protects an `Account` admin route, like updating `AccountSettings`
///
/// This function will check if the request has a valid `x-api-key` header
/// and if the token is the one stored in DB for the `Account`
pub async fn protect_account_route(
    headers: &HeaderMap,
    ctx: &NitteiContext,
) -> Result<Account, NitteiError> {
    let api_key = match headers.get("x-api-key") {
        Some(api_key) => match api_key.to_str() {
            Ok(api_key) => api_key,
            Err(_) => {
                return Err(NitteiError::Unauthorized(
                    "Malformed api key provided".to_string(),
                ))
            }
        },
        None => {
            return Err(NitteiError::Unauthorized(
                "Unable to find api-key in x-api-key header".to_string(),
            ))
        }
    };

    ctx.repos
        .accounts
        .find_by_apikey(api_key)
        .await
        .map_err(|_| NitteiError::InternalError)?
        .ok_or_else(|| {
            NitteiError::Unauthorized("Invalid api-key provided in x-api-key header".to_string())
        })
}

/// Only checks which account the request is connected to.
/// If it cannot decide from the request which account the
/// client belongs to it will return `NitteiError`
pub async fn protect_public_account_route(
    headers: &HeaderMap,
    ctx: &NitteiContext,
) -> Result<Account, NitteiError> {
    match get_nittei_account_header(headers) {
        Some(res) => {
            let account_id = res?;

            ctx.repos
                .accounts
                .find(&account_id)
                .await
                .map_err(|_| NitteiError::InternalError)?
                .ok_or_else(|| {
                    NitteiError::UnidentifiableClient(
                        "Could not find out which account the client belongs to".into(),
                    )
                })
        }
        // No nittei-account header, then check if this is an admin client
        None => protect_account_route(headers, ctx).await,
    }
}

/// Used for account admin routes by checking that account
/// is not modifying a user in another account
pub async fn account_can_modify_user(
    account: &Account,
    user_id: &ID,
    ctx: &NitteiContext,
) -> Result<User, NitteiError> {
    match ctx.repos.users.find(user_id).await {
        Ok(Some(user)) if user.account_id == account.id => Ok(user),
        Ok(_) => Err(NitteiError::NotFound(format!(
            "User with id: {} was not found",
            user_id
        ))),
        Err(_) => Err(NitteiError::InternalError),
    }
}

/// Used for account admin routes by checking that account
/// is not modifying a calendar in another account
pub async fn account_can_modify_calendar(
    account: &Account,
    calendar_id: &ID,
    ctx: &NitteiContext,
) -> Result<Calendar, NitteiError> {
    match ctx.repos.calendars.find(calendar_id).await {
        Ok(Some(cal)) if cal.account_id == account.id => Ok(cal),
        Ok(_) => Err(NitteiError::NotFound(format!(
            "Calendar with id: {} was not found",
            calendar_id
        ))),
        Err(_) => Err(NitteiError::InternalError),
    }
}

/// Used for account admin routes by checking that account
/// is not modifying an event in another account
pub async fn account_can_modify_event(
    account: &Account,
    event_id: &ID,
    ctx: &NitteiContext,
) -> Result<CalendarEvent, NitteiError> {
    match ctx.repos.events.find(event_id).await {
        Ok(Some(event)) if event.account_id == account.id => Ok(event),
        Ok(_) => Err(NitteiError::NotFound(format!(
            "Calendar event with id: {} was not found",
            event_id
        ))),
        Err(_) => Err(NitteiError::InternalError),
    }
}

/// Used for account admin routes by checking that account
/// is not modifying a schedule in another account
pub async fn account_can_modify_schedule(
    account: &Account,
    schedule_id: &ID,
    ctx: &NitteiContext,
) -> Result<Schedule, NitteiError> {
    match ctx.repos.schedules.find(schedule_id).await {
        Ok(Some(schedule)) if schedule.account_id == account.id => Ok(schedule),
        Ok(_) => Err(NitteiError::NotFound(format!(
            "Schedule with id: {} was not found",
            schedule_id
        ))),
        Err(_) => Err(NitteiError::InternalError),
    }
}

#[cfg(test)]
mod test {
    use axum::{body::Body, http::Request};
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use nittei_domain::PEMKey;
    use nittei_infra::setup_context;

    use super::*;

    async fn setup_account(ctx: &NitteiContext) -> Account {
        let account = get_account();
        ctx.repos.accounts.insert(&account).await.unwrap();
        account
    }

    fn get_token(expired: bool, user_id: ID) -> String {
        let priv_key = std::fs::read("./config/test_private_rsa_key.pem").unwrap();
        let exp = if expired {
            100 // year 1970
        } else {
            5609418990073 // year 2147
        };
        let claims = Claims {
            exp,
            iat: 19,
            nittei_user_id: user_id,
            scheduler_policy: None,
        };
        let enc_key = EncodingKey::from_rsa_pem(&priv_key).unwrap();
        encode(&Header::new(Algorithm::RS256), &claims, &enc_key).unwrap()
    }

    fn get_account() -> Account {
        let pub_key = std::fs::read("./config/test_public_rsa_key.crt").unwrap();
        let pub_key = String::from_utf8(pub_key).expect("Valid pem file");
        let pub_key = PEMKey::new(pub_key).unwrap();
        Account {
            public_jwt_key: Some(pub_key),
            ..Default::default()
        }
    }

    #[tokio::main]
    #[test]
    async fn decodes_valid_token_for_existing_user_in_account() {
        let ctx = setup_context().await.unwrap();
        let account = setup_account(&ctx).await;
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let token = get_token(false, user.id.clone());

        let req = Request::builder()
            .header("nittei-account", account.id.to_string())
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let res = protect_route(req.headers(), &ctx).await;
        assert!(res.is_ok());
    }

    #[tokio::main]
    #[test]
    async fn decodes_valid_token_and_rejects_if_user_is_in_different_account() {
        let ctx = setup_context().await.unwrap();
        let account = setup_account(&ctx).await;
        let account2 = setup_account(&ctx).await;
        let user = User::new(account2.id.clone(), None); // user belongs to account2
        ctx.repos.users.insert(&user).await.unwrap();
        // account1 tries to sign a token with user_id that belongs to account2
        let token = get_token(false, user.id.clone());

        let req = Request::builder()
            .header("nittei-account", account.id.to_string())
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let res = protect_route(req.headers(), &ctx).await;
        assert!(res.is_err());
    }

    #[tokio::main]
    #[test]
    async fn rejects_expired_token() {
        let ctx = setup_context().await.unwrap();
        let account = setup_account(&ctx).await;
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let token = get_token(true, user.id.clone());

        let req = Request::builder()
            .header("nittei-account", account.id.to_string())
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let res = protect_route(req.headers(), &ctx).await;
        assert!(res.is_err());
    }

    #[tokio::main]
    #[test]
    async fn rejects_valid_token_without_account_header() {
        let ctx = setup_context().await.unwrap();
        let account = setup_account(&ctx).await;
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let token = get_token(true, user.id.clone());

        let req = Request::builder()
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let res = protect_route(req.headers(), &ctx).await;
        assert!(res.is_err());
    }

    #[tokio::main]
    #[test]
    async fn rejects_valid_token_with_invalid_account_header() {
        let ctx = setup_context().await.unwrap();
        let account = setup_account(&ctx).await;
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();
        let token = get_token(true, user.id.clone());

        let req = Request::builder()
            .header("nittei-account", account.id.to_string() + "s")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let res = protect_route(req.headers(), &ctx).await;
        assert!(res.is_err());
    }

    #[tokio::main]
    #[test]
    async fn rejects_garbage_token_with_valid_account_header() {
        let ctx = setup_context().await.unwrap();
        let _account = setup_account(&ctx).await;
        let token = "sajfosajfposajfopaso12";

        let req = Request::builder()
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let res = protect_route(req.headers(), &ctx).await;
        assert!(res.is_err());
    }

    #[tokio::main]
    #[test]
    async fn rejects_invalid_authz_header() {
        let ctx = setup_context().await.unwrap();
        let account = setup_account(&ctx).await;
        let user = User::new(account.id.clone(), None);
        ctx.repos.users.insert(&user).await.unwrap();

        let req = Request::builder()
            .header("nittei-account", account.id.to_string())
            .header("Authorization", "Bea")
            .body(Body::empty())
            .unwrap();
        let res = protect_route(req.headers(), &ctx).await;
        assert!(res.is_err());
    }

    #[tokio::main]
    #[test]
    async fn rejects_req_without_headers() {
        let ctx = setup_context().await.unwrap();
        let _account = setup_account(&ctx).await;

        let req = Request::builder().body(Body::empty()).unwrap();
        let res = protect_route(req.headers(), &ctx).await;
        assert!(res.is_err());
    }
}
