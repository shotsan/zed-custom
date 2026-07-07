use axum::{
    extract::{State, Json, Query, WebSocketUpgrade},
    extract::ws::{WebSocket, Message as AxumWsMessage},
    http::StatusCode,
    response::{IntoResponse, Redirect, Html},
    routing::{get, post},
    Router,
};
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error, warn};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use reqwest::Client;
use redis::AsyncCommands;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::broadcast;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    id: String,
    user: String,
    text: String,
    timestamp: String,
    #[serde(default)]
    is_incoming: bool,
    team_id: String,
    channel: String,
    target_user: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    sub: String,
    team_id: String,
    exp: usize,
}

#[derive(Clone)]
struct AppState {
    http_client: Client,
    slack_client_id: String,
    slack_client_secret: String,
    slack_app_token: String,
    jwt_secret: String,
    redis_client: redis::Client,
    tx: broadcast::Sender<Message>,
}

#[derive(Deserialize, Debug)]
struct SlackConnectionResponse {
    ok: bool,
    url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SlackSocketMessage {
    envelope_id: String,
    #[serde(rename = "type")]
    msg_type: String,
    payload: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct OauthCallback {
    code: String,
    state: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SlackOauthResponse {
    ok: bool,
    access_token: Option<String>,
    team: Option<SlackTeam>,
    authed_user: Option<SlackUser>,
}

#[derive(Deserialize, Debug)]
struct SlackTeam {
    id: String,
}

#[derive(Deserialize, Debug)]
struct SlackUser {
    id: String,
}

#[derive(Deserialize)]
struct InstallQuery {
    state: Option<String>,
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider().install_default().ok();
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let slack_client_id = std::env::var("SLACK_CLIENT_ID").unwrap_or_default();
    let slack_client_secret = std::env::var("SLACK_CLIENT_SECRET").unwrap_or_default();
    let slack_app_token = std::env::var("SLACK_APP_TOKEN").unwrap_or_default();
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_insecure_jwt_secret_please_change".to_string());
    
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url).unwrap();

    let http_client = Client::new();
    let (tx, _rx) = broadcast::channel(100);

    let state = AppState {
        http_client: http_client.clone(),
        slack_client_id,
        slack_client_secret,
        slack_app_token: slack_app_token.clone(),
        jwt_secret,
        redis_client,
        tx: tx.clone(),
    };

    let app = Router::new()
        .route("/slack/install", get(install_handler))
        .route("/slack/oauth_redirect", get(oauth_callback_handler))
        .route("/api/send_message", post(send_message_handler))
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state.clone());

    // Spawn Slack Socket Mode loop
    let socket_client = http_client.clone();
    let socket_redis = state.redis_client.clone();
    tokio::spawn(async move {
        let mut user_cache: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        if slack_app_token.is_empty() {
            error!("SLACK_APP_TOKEN not set. Skipping Socket Mode.");
            return;
        }

        loop {
            info!("Connecting to Slack Socket Mode...");
            let res = socket_client.post("https://slack.com/api/apps.connections.open")
                .header("Authorization", format!("Bearer {}", slack_app_token))
                .send()
                .await;

            if let Ok(response) = res {
                if let Ok(conn_res) = response.json::<SlackConnectionResponse>().await {
                    if conn_res.ok {
                        if let Some(url) = conn_res.url {
                            info!("WebSocket URL obtained: {}", url);
                            if let Ok((mut ws_stream, _)) = connect_async(&url).await {
                                info!("Connected to Slack WebSocket successfully!");
                                while let Some(msg) = ws_stream.next().await {
                                    if let Ok(WsMessage::Text(text)) = msg {
                                        if let Ok(socket_msg) = serde_json::from_str::<SlackSocketMessage>(&text) {
                                            let ack = serde_json::json!({ "envelope_id": socket_msg.envelope_id });
                                            let _ = ws_stream.send(WsMessage::Text(ack.to_string().into())).await;

                                            if socket_msg.msg_type == "events_api" {
                                                if let Some(payload) = socket_msg.payload {
                                                    if let Some(team_id) = payload.get("team_id").and_then(|t| t.as_str()) {
                                                        if let Some(event) = payload.get("event") {
                                                            if event.get("type").and_then(|t| t.as_str()) == Some("message") {
                                                                let bot_id = event.get("bot_id");
                                                                // Ignore messages sent by bots to prevent loops
                                                                if bot_id.is_none() {
                                                                    let mut text = event.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
                                                                    if text.starts_with('!') {
                                                                        text = format!("/{}", &text[1..]);
                                                                    }
                                                                    let user_id = event.get("user").and_then(|u| u.as_str()).unwrap_or("Unknown").to_string();
                                                                    
                                                                    let mut display_name = user_id.clone();
                                                                    if user_id != "Unknown" {
                                                                        if let Some(cached_name) = user_cache.get(&user_id) {
                                                                            display_name = cached_name.clone();
                                                                        } else {
                                                                            if let Ok(mut con) = socket_redis.get_multiplexed_async_connection().await {
                                                                                let token_key = format!("slack_team_token:{}", team_id);
                                                                                if let Ok(bot_token) = con.get::<_, String>(&token_key).await {
                                                                                    if let Ok(res) = socket_client.get(format!("https://slack.com/api/users.info?user={}", user_id))
                                                                                        .header("Authorization", format!("Bearer {}", bot_token))
                                                                                        .send().await 
                                                                                    {
                                                                                        if let Ok(user_info) = res.json::<serde_json::Value>().await {
                                                                                            let display_name_val = user_info.pointer("/user/profile/display_name").and_then(|n| n.as_str()).filter(|s| !s.is_empty());
                                                                                            let real_name_val = user_info.pointer("/user/profile/real_name").and_then(|n| n.as_str()).filter(|s| !s.is_empty());
                                                                                            let root_real_name = user_info.pointer("/user/real_name").and_then(|n| n.as_str()).filter(|s| !s.is_empty());
                                                                                            let root_name = user_info.pointer("/user/name").and_then(|n| n.as_str()).filter(|s| !s.is_empty());
                                                                                            
                                                                                            if let Some(best_name) = display_name_val.or(real_name_val).or(root_real_name).or(root_name) {
                                                                                                display_name = best_name.to_string();
                                                                                                user_cache.insert(user_id.clone(), best_name.to_string());
                                                                                            } else {
                                                                                                tracing::warn!("Could not find name for user {}. Raw response: {:?}", user_id, user_info);
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }

                                                                    let ts = event.get("ts").and_then(|t| t.as_str()).unwrap_or("Now").to_string();
                                                                    let channel = event.get("channel").and_then(|c| c.as_str()).unwrap_or("Unknown").to_string();

                                                                    let target_user = if channel.starts_with('D') {
                                                                        Some(user_id.clone())
                                                                    } else {
                                                                        None
                                                                    };

                                                                    let msg = Message {
                                                                        id: ts.clone(),
                                                                        user: display_name,
                                                                        text,
                                                                        timestamp: ts,
                                                                        is_incoming: true,
                                                                        team_id: team_id.to_string(),
                                                                        channel,
                                                                        target_user,
                                                                    };

                                                                    // Broadcast to any connected Zed clients
                                                                    let _ = tx.send(msg);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        error!("Failed to get Slack websocket URL. Not OK.");
                    }
                }
            } else {
                error!("Failed to request apps.connections.open");
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    // Determine port, Azure sets PORT environment variable
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Axum Microservice listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn install_handler(State(state): State<AppState>) -> Redirect {
    let rand_state: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // Store state in Redis for 10 minutes (600s) to prevent CSRF
    if let Ok(mut con) = state.redis_client.get_multiplexed_async_connection().await {
        let _: redis::RedisResult<()> = con.set_ex(format!("oauth_state:{}", rand_state), "valid", 600).await;
    }

    let url = format!(
        "https://slack.com/oauth/v2/authorize?client_id={}&scope=chat:write,chat:write.customize,channels:history,users:read&state={}",
        state.slack_client_id, rand_state
    );
    Redirect::to(&url)
}

async fn oauth_callback_handler(
    State(state): State<AppState>,
    Query(query): Query<OauthCallback>,
) -> impl IntoResponse {
    let csrf_state = match query.state {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, Html("Missing state parameter".to_string())),
    };

    let mut con = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Redis Error".to_string())),
    };

    let state_val: redis::RedisResult<String> = con.get(format!("oauth_state:{}", csrf_state)).await;
    if state_val.is_err() {
        return (StatusCode::FORBIDDEN, Html("Invalid or expired state parameter (CSRF protection)".to_string()));
    }
    // Delete state so it can't be reused
    let _: redis::RedisResult<()> = con.del(format!("oauth_state:{}", csrf_state)).await;

    let res = state.http_client.post("https://slack.com/api/oauth.v2.access")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("client_id={}&client_secret={}&code={}", state.slack_client_id, state.slack_client_secret, query.code))
        .send()
        .await;

    if let Ok(response) = res {
        let raw_body = response.text().await.unwrap_or_default();
        tracing::info!("Slack OAuth Response: {}", raw_body);
        if let Ok(oauth_res) = serde_json::from_str::<SlackOauthResponse>(&raw_body) {
            if oauth_res.ok {
                if let (Some(token), Some(team), Some(user)) = (&oauth_res.access_token, &oauth_res.team, &oauth_res.authed_user) {
                    // Save access token securely in Redis tied to Team and User ID
                    let _ : redis::RedisResult<()> = con.set(format!("slack_token:{}:{}", team.id, user.id), token).await;
                    let _ : redis::RedisResult<()> = con.set(format!("slack_team_token:{}", team.id), token).await;

                    // Generate JWT for Zed Client
                    let claims = Claims {
                        sub: user.id.clone(),
                        team_id: team.id.clone(),
                        exp: (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 60 * 60 * 24 * 30) as usize, // 30 days
                    };

                    let jwt = encode(&Header::default(), &claims, &EncodingKey::from_secret(state.jwt_secret.as_bytes())).unwrap();

                    let html = format!(

                        r##"
                        <!DOCTYPE html>
                        <html lang="en">
                        <head>
                            <meta charset="UTF-8">
                            <meta name="viewport" content="width=device-width, initial-scale=1.0">
                            <title>Zed Slack Authentication</title>
                            <style>
                                :root {{
                                    --bg-color: #1a1a1a;
                                    --panel-bg: #222222;
                                    --border-color: #333333;
                                    --text-primary: #e0e0e0;
                                    --text-secondary: #888888;
                                    --accent-color: #4da6ff;
                                    --accent-hover: #73bfff;
                                    --success-color: #66cc66;
                                }}
                                body {{
                                    background-color: var(--bg-color);
                                    color: var(--text-primary);
                                    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, "Open Sans", "Helvetica Neue", sans-serif;
                                    display: flex;
                                    justify-content: center;
                                    align-items: center;
                                    height: 100vh;
                                    margin: 0;
                                    -webkit-font-smoothing: antialiased;
                                }}
                                .container {{
                                    background-color: var(--panel-bg);
                                    border: 1px solid var(--border-color);
                                    border-radius: 12px;
                                    padding: 48px;
                                    width: 100%;
                                    max-width: 480px;
                                    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
                                    text-align: center;
                                }}
                                h2 {{
                                    margin-top: 0;
                                    color: var(--text-primary);
                                    font-size: 24px;
                                    font-weight: 600;
                                    letter-spacing: -0.5px;
                                }}
                                p {{
                                    line-height: 1.6;
                                    margin-bottom: 32px;
                                    color: var(--text-secondary);
                                    font-size: 15px;
                                }}
                                .token-wrapper {{
                                    background-color: #111111;
                                    border: 1px solid var(--border-color);
                                    border-radius: 8px;
                                    padding: 16px;
                                    position: relative;
                                    margin-bottom: 32px;
                                    text-align: left;
                                }}
                                .token-label {{
                                    font-size: 12px;
                                    text-transform: uppercase;
                                    letter-spacing: 1px;
                                    color: var(--text-secondary);
                                    margin-bottom: 8px;
                                    display: block;
                                }}
                                textarea {{
                                    width: 100%;
                                    box-sizing: border-box;
                                    height: 80px;
                                    background-color: transparent;
                                    color: #9cdcfe;
                                    border: none;
                                    font-family: "SF Mono", "Fira Code", Consolas, Monaco, monospace;
                                    font-size: 13px;
                                    line-height: 1.5;
                                    resize: none;
                                    outline: none;
                                }}
                                button {{
                                    background-color: var(--accent-color);
                                    color: #111;
                                    border: none;
                                    border-radius: 6px;
                                    padding: 12px 32px;
                                    font-size: 15px;
                                    font-weight: 600;
                                    cursor: pointer;
                                    transition: all 0.2s ease;
                                    width: 100%;
                                    display: flex;
                                    justify-content: center;
                                    align-items: center;
                                    gap: 8px;
                                }}
                                button:hover {{
                                    background-color: var(--accent-hover);
                                    transform: translateY(-1px);
                                }}
                                button:active {{
                                    transform: translateY(1px);
                                }}
                                .logo {{
                                    width: 48px;
                                    height: 48px;
                                    background-color: var(--accent-color);
                                    border-radius: 12px;
                                    display: inline-flex;
                                    justify-content: center;
                                    align-items: center;
                                    margin-bottom: 24px;
                                    color: #111;
                                    font-size: 24px;
                                    font-weight: bold;
                                }}
                                .secure-badge {{
                                    display: inline-flex;
                                    align-items: center;
                                    gap: 6px;
                                    font-size: 13px;
                                    color: var(--success-color);
                                    margin-bottom: 24px;
                                    background: rgba(102, 204, 102, 0.1);
                                    padding: 6px 12px;
                                    border-radius: 16px;
                                }}
                            </style>
                        </head>
                        <body>
                            <div class="container">
                                <div class="logo">Z</div>
                                <h2>Connection Successful</h2>
                                <div class="secure-badge">
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
                                    Secure Authentication
                                </div>
                                <p>Your Slack workspace is successfully linked. Copy the secure access token below and paste it into the Zed Slack panel to complete the setup.</p>
                                <div class="token-wrapper">
                                    <span class="token-label">Access Token</span>
                                    <textarea id="token" readonly spellcheck="false" onclick="this.select()">{}</textarea>
                                </div>
                                <button id="copyBtn" onclick="copyToken()">
                                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                                    Copy Token to Clipboard
                                </button>
                            </div>
                            <script>
                                function copyToken() {{
                                    var copyText = document.getElementById("token");
                                    copyText.select();
                                    document.execCommand("copy");
                                    var btn = document.getElementById("copyBtn");
                                    var originalHtml = btn.innerHTML;
                                    btn.innerHTML = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg> Token Copied!';
                                    btn.style.backgroundColor = "var(--success-color)";
                                    btn.style.color = "#111";
                                    setTimeout(function() {{
                                        btn.innerHTML = originalHtml;
                                        btn.style.backgroundColor = "var(--accent-color)";
                                    }}, 3000);
                                }}
                            </script>
                        </body>
                        </html>
                        "##,
                        jwt
                    );
                    return (StatusCode::OK, Html(html));
                } else {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Slack returned ok=true but missing fields. token: {:?}, team: {:?}, user: {:?}", oauth_res.access_token, oauth_res.team, oauth_res.authed_user)));
                }
            } else {
                return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Slack returned error: {:?}", oauth_res)));
            }
        } else {
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Failed to parse Slack response: {}", raw_body)));
        }
    } else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Reqwest failed to connect to Slack.")));
    }
}

#[derive(Deserialize)]
struct SendMessageRequest {
    channel: String,
    text: String,
}

// Helper function to extract and verify JWT
fn verify_jwt(headers: &HeaderMap, secret: &str) -> Result<Claims, StatusCode> {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            let token = &auth[7..];
            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &Validation::default(),
            ).map_err(|_| StatusCode::UNAUTHORIZED)?;
            return Ok(token_data.claims);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

async fn send_message_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SendMessageRequest>,
) -> impl IntoResponse {
    // 1. Verify JWT
    let claims = match verify_jwt(&headers, &state.jwt_secret) {
        Ok(c) => c,
        Err(status) => return status.into_response(),
    };

    // 2. Get token from Redis using identity from JWT
    let mut con = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let token_key = format!("slack_token:{}:{}", claims.team_id, claims.sub);
    let token: redis::RedisResult<String> = con.get(&token_key).await;
    let bot_token = match token {
        Ok(t) => t,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    // 3. Send Message
    let mut formatted_text = payload.text;
    formatted_text = formatted_text.replace("## Assistant", "*Coder*").replace("## assistant", "*Coder*");
    formatted_text = formatted_text.replace("## User", "*User*").replace("## user", "*User*");

    let req_body = serde_json::json!({
        "channel": payload.channel,
        "text": formatted_text,
        "username": "Zed",
        "icon_url": "https://raw.githubusercontent.com/zed-industries/zed/main/crates/zed/resources/app-icon@2x.png"
    });

    let res = state.http_client.post("https://slack.com/api/chat.postMessage")
        .header("Authorization", format!("Bearer {}", bot_token))
        .json(&req_body)
        .send()
        .await;

    match res {
        Ok(response) => {
            if let Ok(body_str) = response.text().await {
                tracing::info!("Slack postMessage response: {}", body_str);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
                    if json.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                        return StatusCode::OK.into_response();
                    } else {
                        let err = json.get("error").and_then(|v| v.as_str()).unwrap_or("unknown_error");
                        return (StatusCode::BAD_REQUEST, err.to_string()).into_response();
                    }
                }
            }
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        },
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Verify JWT from query string
    let claims = match decode::<Claims>(
        &query.token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(c) => c.claims,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    ws.on_upgrade(move |socket| handle_socket(socket, state, claims))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, claims: Claims) {
    let mut rx = state.tx.subscribe();
    
    // We only care about sending messages down to Zed based on its team_id
    while let Ok(msg) = rx.recv().await {
        if msg.team_id == claims.team_id {
            // Security check: If this is a DM, ONLY send it to the exact user involved
            if let Some(target) = &msg.target_user {
                if target != &claims.sub {
                    continue; // Drop the message, it is a private DM for someone else
                }
            }

            if let Ok(json) = serde_json::to_string(&msg) {
                if socket.send(AxumWsMessage::Text(json)).await.is_err() {
                    break; // Client disconnected
                }
            }
        }
    }
}
