use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[derive(Clone)]
struct AppState {
    http_client: Client,
    centrifugo_api_key: String,
    centrifugo_url: String,
}

#[derive(Deserialize, Debug)]
struct SlackEventPayload {
    // Basic Slack Event Structure
    #[serde(rename = "type")]
    event_type: String,
    challenge: Option<String>,
    event: Option<SlackInnerEvent>,
}

#[derive(Deserialize, Debug)]
struct SlackInnerEvent {
    #[serde(rename = "type")]
    event_type: String,
    user: Option<String>,
    text: Option<String>,
    channel: Option<String>,
}

#[derive(Serialize)]
struct CentrifugoPublishRequest {
    channel: String,
    data: serde_json::Value,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        http_client: Client::new(),
        centrifugo_api_key: "api_key".to_string(),
        centrifugo_url: "http://localhost:8000/api".to_string(),
    };

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/slack/events", post(handle_slack_event))
        .route("/api/messages", post(handle_outgoing_message))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handle_slack_event(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SlackEventPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("Received Slack event: {:?}", payload);

    // Handle Slack URL Verification (Challenge)
    if payload.event_type == "url_verification" {
        if let Some(challenge) = payload.challenge {
            return Ok(Json(serde_json::json!({ "challenge": challenge })));
        }
    }

    // TODO: Verify HMAC Signature here (requires Slack Signing Secret)

    if let Some(event) = payload.event {
        if event.event_type == "message" && event.text.is_some() {
            // Forward to Centrifugo
            let channel_name = event.channel.unwrap_or_else(|| "general".to_string());
            let pub_req = CentrifugoPublishRequest {
                channel: format!("slack:{}", channel_name),
                data: serde_json::json!({
                    "user": event.user.unwrap_or_else(|| "unknown".to_string()),
                    "text": event.text.unwrap(),
                }),
            };

            let res = state
                .http_client
                .post(&state.centrifugo_url)
                .header("Authorization", format!("apikey {}", state.centrifugo_api_key))
                .json(&serde_json::json!({
                    "method": "publish",
                    "params": pub_req
                }))
                .send()
                .await;

            if let Err(e) = res {
                tracing::error!("Failed to publish to Centrifugo: {}", e);
            }
        }
    }

    Ok(Json(serde_json::json!({"status": "ok"})))
}

#[derive(Deserialize, Debug)]
struct OutgoingMessage {
    channel: String,
    text: String,
}

async fn handle_outgoing_message(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<OutgoingMessage>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Received outgoing message request: {:?}", payload);
    
    // TODO: Look up user's Slack token from Keycloak/DB
    // TODO: POST to https://slack.com/api/chat.postMessage

    Ok(StatusCode::OK)
}
