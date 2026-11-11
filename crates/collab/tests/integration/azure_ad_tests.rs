use crate::test_server::TestServer;
use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use std::sync::Arc;
use base64::prelude::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_azure_ad_tier4_missing_config() {
    let mut server = TestServer::start().await;

    // Login without config
    let req = Request::builder()
        .uri("/auth/login")
        .body(Body::empty())
        .unwrap();
    let response = collab::api::routes(server.server.clone())
        .layer(axum::extract::Extension(server.app_state.clone()))
        .oneshot(req)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Callback without config
    let req = Request::builder()
        .uri("/auth/callback?code=abc")
        .body(Body::empty())
        .unwrap();
    let response = collab::api::routes(server.server.clone())
        .layer(axum::extract::Extension(server.app_state.clone()))
        .oneshot(req)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_azure_ad_tier1_login_redirect() {
    let mut server = TestServer::start().await;

    let mut config = server.app_state.config.clone();
    config.azure_ad_client_id = Some("test_client".into());
    config.azure_ad_tenant_id = Some("test_tenant".into());

    let mut app_state_mut = (*server.app_state).clone();
    app_state_mut.config = config;
    let new_app_state = Arc::new(app_state_mut);

    let req = Request::builder()
        .uri("/auth/login?state=mystate")
        .body(Body::empty())
        .unwrap();

    let response = collab::api::routes(server.server.clone())
        .layer(axum::extract::Extension(new_app_state))
        .oneshot(req)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = response.headers().get("location").unwrap().to_str().unwrap();
    assert!(location.starts_with("https://login.microsoftonline.com/test_tenant/oauth2/v2.0/authorize"));
    assert!(location.contains("client_id=test_client"));
    assert!(location.contains("state=mystate"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_azure_ad_tier2_callback_success() {
    let mut server = TestServer::start().await;

    // Spawn a dummy Axum server to act as Azure AD
    let mock_ad = axum::Router::new().route(
        "/test_tenant/oauth2/v2.0/token",
        axum::routing::post(|axum::extract::Form(form): axum::extract::Form<std::collections::HashMap<String, String>>| async move {
            assert_eq!(form.get("client_id").unwrap(), "test_client");
            assert_eq!(form.get("client_secret").unwrap(), "test_secret");
            assert_eq!(form.get("code").unwrap(), "test_code");
            
            let payload = serde_json::json!({
                "sub": "user_123",
                "email": "test@example.com"
            });
            let payload_b64 = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_string(&payload).unwrap());
            let id_token = format!("header.{}.signature", payload_b64);
            
            axum::Json(serde_json::json!({
                "access_token": "mock_ad_token",
                "id_token": id_token
            }))
        }),
    );

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(mock_ad.into_make_service())
            .await
            .unwrap();
    });

    let mut config = server.app_state.config.clone();
    config.azure_ad_client_id = Some("test_client".into());
    config.azure_ad_client_secret = Some("test_secret".into());
    config.azure_ad_tenant_id = Some("test_tenant".into());
    config.azure_ad_url = Some(format!("http://127.0.0.1:{}", port));

    let mut app_state_mut = (*server.app_state).clone();
    app_state_mut.config = config;
    let new_app_state = Arc::new(app_state_mut);

    let req = Request::builder()
        .uri("/auth/callback?code=test_code&state=mystate")
        .body(Body::empty())
        .unwrap();

    let response = collab::api::routes(server.server.clone())
        .layer(axum::extract::Extension(new_app_state.clone()))
        .oneshot(req)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = response.headers().get("location").unwrap().to_str().unwrap();
    assert!(location.starts_with("zed://login?access_token="));

    // Verify user was created in DB
    let user = new_app_state.db.get_user_by_github_login("test@example.com").await.unwrap().unwrap();
    
    // Check stable hashing for `sub`
    let mut hasher = sha2::Sha256::new();
    hasher.update(b"user_123");
    let result = hasher.finalize();
    let expected_github_user_id = i32::from_le_bytes(result[0..4].try_into().unwrap());
    
    assert_eq!(user.github_login, "test@example.com");
    assert_eq!(user.github_user_id, expected_github_user_id);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_azure_ad_tier3_callback_errors() {
    let mut server = TestServer::start().await;

    let mut config = server.app_state.config.clone();
    config.azure_ad_client_id = Some("test_client".into());
    config.azure_ad_client_secret = Some("test_secret".into());
    config.azure_ad_tenant_id = Some("test_tenant".into());

    let mut app_state_mut = (*server.app_state).clone();
    app_state_mut.config = config;
    let new_app_state = Arc::new(app_state_mut);

    // Error param present
    let req = Request::builder()
        .uri("/auth/callback?error=access_denied")
        .body(Body::empty())
        .unwrap();

    let response = collab::api::routes(server.server.clone())
        .layer(axum::extract::Extension(new_app_state.clone()))
        .oneshot(req)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Missing code
    let req = Request::builder()
        .uri("/auth/callback?state=mystate")
        .body(Body::empty())
        .unwrap();

    let response = collab::api::routes(server.server.clone())
        .layer(axum::extract::Extension(new_app_state.clone()))
        .oneshot(req)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
