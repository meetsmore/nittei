use chrono::Utc;
use nittei_domain::{IntegrationProvider, User};
use serde::Deserialize;
use tracing::error;

use crate::{CodeTokenRequest, CodeTokenResponse, NitteiContext};

// https://developers.google.com/identity/protocols/oauth2/web-server#httprest_3

const TOKEN_REFETCH_ENDPOINT: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const CODE_TOKEN_EXCHANGE_ENDPOINT: &str =
    "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const REQUIRED_OAUTH_SCOPES: [&str; 2] = [
    "https://graph.microsoft.com/calendars.readwrite",
    "offline_access",
];

// https://docs.microsoft.com/en-us/graph/auth-v2-user#request
struct RefreshTokenRequest {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    refresh_token: String,
    scope: String,
}

// https://docs.microsoft.com/en-us/graph/auth-v2-user#response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RefreshTokenResponse {
    refresh_token: String,
    access_token: String,
    scope: String,
    token_type: String,
    // Access token expires in specified in seconds
    expires_in: i64,
}

async fn refresh_access_token(req: RefreshTokenRequest) -> anyhow::Result<RefreshTokenResponse> {
    let params = [
        ("client_id", req.client_id.as_str()),
        ("client_secret", req.client_secret.as_str()),
        ("redirect_uri", req.redirect_uri.as_str()),
        ("refresh_token", req.refresh_token.as_str()),
        ("scope", req.scope.as_str()),
        ("grant_type", "refresh_token"),
    ];
    let client = reqwest::Client::new();
    let res = client
        .post(TOKEN_REFETCH_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!(
                "[Network Error] Outlook OAuth refresh token failed with error: {:?}",
                e
            );

            e
        })?;

    res.json::<RefreshTokenResponse>().await.map_err(|e| {
        error!(
            "[Unexpected Response] Google OAuth refresh token failed with error: {:?}",
            e
        );

        anyhow::Error::new(e)
    })
}

pub async fn exchange_code_token(req: CodeTokenRequest) -> Result<CodeTokenResponse, ()> {
    let params = [
        ("client_id", req.client_id.as_str()),
        ("client_secret", req.client_secret.as_str()),
        ("redirect_uri", req.redirect_uri.as_str()),
        ("code", req.code.as_str()),
        ("scope", &REQUIRED_OAUTH_SCOPES.join(" ")),
        ("grant_type", "authorization_code"),
    ];

    let client = reqwest::Client::new();

    let res = client
        .post(CODE_TOKEN_EXCHANGE_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!(
                "[Network Error] Outlook OAuth code token exchange failed with error: {:?}",
                e
            );
        })?;

    let res = res.json::<CodeTokenResponse>().await.map_err(|e| {
        error!(
            "[Unexpected Response] Outlook OAuth code token exchange failed with error: {:?}",
            e
        );
    })?;

    let scopes = res
        .scope
        .split(' ')
        .map(|scope| scope.to_lowercase())
        .collect::<Vec<_>>();
    for required_scope in REQUIRED_OAUTH_SCOPES.iter() {
        if required_scope == &"offline_access" {
            continue;
        }
        if !scopes.contains(&required_scope.to_string()) {
            error!(
                "[Missing scopes] Outlook OAuth code token exchange failed. Missing scope: {:?}, got the following scopes: {:?}",
                required_scope, scopes
            );

            return Err(());
        }
    }

    Ok(res)
}

pub async fn get_access_token(user: &User, ctx: &NitteiContext) -> Option<String> {
    // Check if user has connected to outlook
    let mut integrations = ctx.repos.user_integrations.find(&user.id).await.ok()?;
    let integration = integrations
        .iter_mut()
        .find(|i| matches!(i.provider, IntegrationProvider::Outlook))?;

    let now = Utc::now().timestamp_millis();
    let one_minute_in_millis = 1000 * 60;
    if now + one_minute_in_millis <= integration.access_token_expires_ts {
        // Current access token is still valid for at least one minutes so return it
        return Some(integration.access_token.clone());
    }
    // Access token has or will expire soon, now renew it

    // The account contains the google client id and secret
    let acc_integrations = match ctx.repos.account_integrations.find(&user.account_id).await {
        Ok(acc_integrations) => acc_integrations,
        Err(_) => return None,
    };
    let outlook_settings = acc_integrations
        .into_iter()
        .find(|i| matches!(i.provider, IntegrationProvider::Outlook))?;

    let refresh_token_req = RefreshTokenRequest {
        client_id: outlook_settings.client_id,
        client_secret: outlook_settings.client_secret,
        refresh_token: integration.refresh_token.clone(),
        redirect_uri: outlook_settings.redirect_uri.clone(),
        scope: REQUIRED_OAUTH_SCOPES.join(" "),
    };
    let data = refresh_access_token(refresh_token_req).await;
    match data {
        Ok(tokens) => {
            integration.access_token = tokens.access_token;
            let now = Utc::now().timestamp_millis();
            let expires_in_millis = tokens.expires_in * 1000;
            integration.access_token_expires_ts = now + expires_in_millis;
            let access_token = integration.access_token.clone();

            // Update user with updated google tokens
            if let Err(e) = ctx.repos.user_integrations.save(integration).await {
                error!(
                    "Unable to save updated google credentials for user: {}. Error: {:?}",
                    user.id, e
                );
            }

            // Return access_token
            Some(access_token)
        }
        Err(e) => {
            error!(
                "Unable to refresh outlook oauth access token for user: {}. Error: {:?}",
                user.id, e
            );
            None
        }
    }
}
