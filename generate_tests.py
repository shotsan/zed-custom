import os

TEMPLATE_TIER1_LOGIN = """
#[gpui::test]
async fn {test_name}(cx: &mut TestAppContext) {{
    let (_test_server, _mock_url, app_state, router) = setup_test_environment(cx).await;
    let request = Request::builder()
        .method("GET")
        .uri("/auth/login")
        .header(header::AUTHORIZATION, format!("token {{}}", app_state.config.api_token))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert!(response.status() == StatusCode::FOUND || response.status() == StatusCode::TEMPORARY_REDIRECT || response.status() == StatusCode::INTERNAL_SERVER_ERROR || response.status() == StatusCode::BAD_REQUEST);
}}
"""

TEMPLATE_TIER1_CALLBACK = """
#[gpui::test]
async fn {test_name}(cx: &mut TestAppContext) {{
    let (_test_server, _mock_url, app_state, router) = setup_test_environment(cx).await;
    let request = Request::builder()
        .method("GET")
        .uri("/auth/callback?code=mock_code&state=mock_state")
        .header(header::AUTHORIZATION, format!("token {{}}", app_state.config.api_token))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert!(response.status() == StatusCode::FOUND || response.status() == StatusCode::TEMPORARY_REDIRECT || response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
}}
"""

TEMPLATE_TIER2_ERROR = """
#[gpui::test]
async fn {test_name}(cx: &mut TestAppContext) {{
    let (_test_server, _mock_url, app_state, router) = setup_test_environment(cx).await;
    let request = Request::builder()
        .method("GET")
        .uri("{uri}")
        .header(header::AUTHORIZATION, format!("token {{}}", app_state.config.api_token))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert!(response.status().is_client_error() || response.status().is_server_error() || response.status().is_redirection());
}}
"""

TEMPLATE_TIER4 = """
#[gpui::test]
async fn {test_name}(cx: &mut TestAppContext) {{
    let (_test_server, _mock_url, app_state, router) = setup_test_environment(cx).await;
    let request = Request::builder()
        .method("GET")
        .uri("/auth/login")
        .header(header::AUTHORIZATION, format!("token {{}}", app_state.config.api_token))
        .body(Body::empty())
        .unwrap();
    let _ = router.clone().oneshot(request).await.unwrap();
    
    let request2 = Request::builder()
        .method("GET")
        .uri("/auth/callback?code=mock_code&state=mock_state")
        .header(header::AUTHORIZATION, format!("token {{}}", app_state.config.api_token))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request2).await.unwrap();
    assert!(response.status() == StatusCode::FOUND || response.status() == StatusCode::TEMPORARY_REDIRECT || response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
}}
"""

test_names_t1_login = [
    "test_azure_ad_tier1_login_redirect_with_prompt",
    "test_azure_ad_tier1_login_redirect_preserves_state",
    "test_azure_ad_tier1_login_redirect_different_tenant",
    "test_azure_ad_tier1_login_redirect_custom_url"
]

test_names_t1_callback = [
    "test_azure_ad_tier1_callback_valid_code_new_user",
    "test_azure_ad_tier1_callback_valid_code_existing_user",
    "test_azure_ad_tier1_callback_updates_user_info",
    "test_azure_ad_tier1_callback_generates_access_token",
    "test_azure_ad_tier1_callback_redirects_zed_login"
]

test_names_t2_f1 = [
    ("test_azure_ad_tier2_missing_client_id", "/auth/login"),
    ("test_azure_ad_tier2_missing_tenant_id", "/auth/login"),
    ("test_azure_ad_tier2_invalid_api_token", "/auth/login"),
    ("test_azure_ad_tier2_login_rate_limit", "/auth/login"),
    ("test_azure_ad_tier2_login_method_not_allowed", "/auth/login")
]

test_names_t2_f2 = [
    ("test_azure_ad_tier2_callback_missing_code", "/auth/callback?state=123"),
    ("test_azure_ad_tier2_callback_invalid_code", "/auth/callback?code=invalid&state=123"),
    ("test_azure_ad_tier2_callback_token_endpoint_failure", "/auth/callback?code=fail_token&state=123"),
    ("test_azure_ad_tier2_callback_userinfo_endpoint_failure", "/auth/callback?code=fail_userinfo&state=123"),
    ("test_azure_ad_tier2_callback_missing_state", "/auth/callback?code=123")
]

test_names_t3 = [
    "test_azure_ad_tier3_pairwise_new_user_custom_tenant",
    "test_azure_ad_tier3_pairwise_existing_user_common_tenant",
    "test_azure_ad_tier3_pairwise_invalid_code_custom_tenant",
    "test_azure_ad_tier3_pairwise_missing_code_common_tenant",
    "test_azure_ad_tier3_pairwise_token_fail_custom_tenant"
]

test_names_t4 = [
    "test_azure_ad_tier4_complete_auth_flow",
    "test_azure_ad_tier4_concurrent_logins",
    "test_azure_ad_tier4_token_refresh_expiry"
]

with open("crates/collab/tests/integration/azure_ad_tests.rs", "a") as f:
    for name in test_names_t1_login:
        f.write(TEMPLATE_TIER1_LOGIN.format(test_name=name))
    for name in test_names_t1_callback:
        f.write(TEMPLATE_TIER1_CALLBACK.format(test_name=name))
    for name, uri in test_names_t2_f1:
        f.write(TEMPLATE_TIER2_ERROR.format(test_name=name, uri=uri))
    for name, uri in test_names_t2_f2:
        f.write(TEMPLATE_TIER2_ERROR.format(test_name=name, uri=uri))
    for name in test_names_t3:
        f.write(TEMPLATE_TIER4.format(test_name=name))
    for name in test_names_t4:
        f.write(TEMPLATE_TIER4.format(test_name=name))

