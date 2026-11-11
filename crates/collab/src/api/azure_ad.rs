use crate::{AppState, Error, Result, auth};
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use base64::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct LoginQueryParams {
    state: Option<String>,
}

#[derive(Deserialize)]
pub struct CallbackQueryParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    code: &'a str,
    redirect_uri: &'a str,
    grant_type: &'a str,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TokenResponse {
    access_token: String,
    id_token: Option<String>,
}

#[derive(Deserialize)]
struct IdTokenPayload {
    sub: String,
    preferred_username: Option<String>,
    email: Option<String>,
}

pub async fn login(
    Query(params): Query<LoginQueryParams>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse> {
    let client_id = app_state
        .config
        .azure_ad_client_id
        .as_deref()
        .ok_or_else(|| Error::Http(StatusCode::INTERNAL_SERVER_ERROR, "Missing azure_ad_client_id".to_string(), Default::default()))?;
    
    let tenant_id = app_state
        .config
        .azure_ad_tenant_id
        .as_deref()
        .unwrap_or("common");

    let base_url = app_state
        .config
        .azure_ad_url
        .as_deref()
        .unwrap_or("https://login.microsoftonline.com");

    let state = params.state.unwrap_or_else(|| {
        let mut state_bytes = [0u8; 16];
        rand::rng().fill(&mut state_bytes);
        BASE64_URL_SAFE_NO_PAD.encode(&state_bytes)
    });

    let redirect_uri = format!("{}/auth/callback", app_state.config.zed_dot_dev_url());

    let url = if let (Some(b2c_tenant), Some(b2c_policy)) = (
        app_state.config.azure_b2c_tenant_name.as_deref(),
        app_state.config.azure_b2c_policy_name.as_deref(),
    ) {
        format!(
            "https://{}.b2clogin.com/{}.onmicrosoft.com/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=openid profile email&state={}",
            b2c_tenant,
            b2c_tenant,
            b2c_policy,
            client_id,
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&state)
        )
    } else {
        format!(
            "{}/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=openid profile email&state={}",
            base_url,
            tenant_id,
            client_id,
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&state)
        )
    };

    Ok(Redirect::temporary(&url))
}

pub async fn callback(
    Query(params): Query<CallbackQueryParams>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse> {
    if params.error.is_some() || params.code.is_none() {
        return Err(Error::Http(StatusCode::BAD_REQUEST, "Invalid callback".to_string(), Default::default()));
    }
    
    let code = params.code.unwrap();

    let client_id = app_state
        .config
        .azure_ad_client_id
        .as_deref()
        .ok_or_else(|| Error::Http(StatusCode::INTERNAL_SERVER_ERROR, "Missing azure_ad_client_id".to_string(), Default::default()))?;
    
    let client_secret = app_state
        .config
        .azure_ad_client_secret
        .as_deref()
        .ok_or_else(|| Error::Http(StatusCode::INTERNAL_SERVER_ERROR, "Missing azure_ad_client_secret".to_string(), Default::default()))?;

    let tenant_id = app_state
        .config
        .azure_ad_tenant_id
        .as_deref()
        .unwrap_or("common");

    let base_url = app_state
        .config
        .azure_ad_url
        .as_deref()
        .unwrap_or("https://login.microsoftonline.com");

    let redirect_uri = format!("{}/auth/callback", app_state.config.zed_dot_dev_url());

    let token_url = if let (Some(b2c_tenant), Some(b2c_policy)) = (
        app_state.config.azure_b2c_tenant_name.as_deref(),
        app_state.config.azure_b2c_policy_name.as_deref(),
    ) {
        format!(
            "https://{}.b2clogin.com/{}.onmicrosoft.com/{}/oauth2/v2.0/token",
            b2c_tenant, b2c_tenant, b2c_policy
        )
    } else {
        format!("{}/{}/oauth2/v2.0/token", base_url, tenant_id)
    };

    let client = reqwest::Client::new();
    let res = client.post(&token_url)
        .form(&TokenRequest {
            client_id,
            client_secret,
            code: &code,
            redirect_uri: &redirect_uri,
            grant_type: "authorization_code",
        })
        .send()
        .await
        .map_err(|e| Error::Http(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch token: {}", e), Default::default()))?;

    if !res.status().is_success() {
        return Err(Error::Http(StatusCode::INTERNAL_SERVER_ERROR, format!("Token endpoint error: {}", res.status()), Default::default()));
    }

    let token_data: TokenResponse = res.json().await
        .map_err(|e| Error::Http(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse token response: {}", e), Default::default()))?;

    let id_token = token_data.id_token.ok_or_else(|| Error::Http(StatusCode::INTERNAL_SERVER_ERROR, "No id_token in response".to_string(), Default::default()))?;

    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 {
        return Err(Error::Http(StatusCode::BAD_REQUEST, "Invalid id_token format".to_string(), Default::default()));
    }

    let payload_json = BASE64_URL_SAFE_NO_PAD.decode(parts[1])
        .or_else(|_| BASE64_STANDARD_NO_PAD.decode(parts[1]))
        .or_else(|_| BASE64_URL_SAFE.decode(parts[1]))
        .or_else(|_| BASE64_STANDARD.decode(parts[1]))
        .map_err(|e| Error::Http(StatusCode::BAD_REQUEST, format!("Failed to decode id_token: {}", e), Default::default()))?;
    
    let payload: IdTokenPayload = serde_json::from_slice(&payload_json)
        .map_err(|e| Error::Http(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse id_token payload: {}", e), Default::default()))?;

    let mut hasher = sha2::Sha256::new();
    hasher.update(payload.sub.as_bytes());
    let result = hasher.finalize();
    let github_user_id = i32::from_le_bytes(result[0..4].try_into().unwrap());
    
    let email = payload.email.or(payload.preferred_username).unwrap_or_else(|| "unknown@example.com".to_string());
    let github_login = email.clone();

    let user = app_state.db.create_user(
        &email,
        None,
        false, // admin
        crate::db::NewUserParams {
            github_login,
            github_user_id,
        }
    ).await?;

    let access_token = auth::create_access_token(&app_state.db, user.user_id, None).await?;
    
    let encrypted_token = if let Some(state) = params.state {
        auth::encrypt_access_token(&access_token, state).unwrap_or(access_token)
    } else {
        access_token
    };

    Ok(Redirect::temporary(&format!("zed://login?access_token={}", urlencoding::encode(&encrypted_token))))
}
