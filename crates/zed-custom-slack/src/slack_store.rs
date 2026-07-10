use futures::StreamExt;
use gpui::{App, Context, Entity, Global, AppContext, WeakEntity, EventEmitter, Task};
use workspace::Workspace;
use collab_ui::channel_view::ChannelView;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SlackMessage {
    pub id: String,
    pub user: String,
    pub text: String,
    #[serde(default)]
    pub is_incoming: bool,
    pub timestamp: String,
    pub channel: Option<String>,
}

#[derive(PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

pub enum SlackEvent {
    MessageReceived(SlackMessage),
}

pub struct SlackStore {
    messages: Vec<SlackMessage>,
    connection_state: ConnectionState,
    workspace: Option<WeakEntity<Workspace>>,
    channel_buffer_subscription: Option<gpui::Subscription>,
    channel_buffer: Option<Entity<channel::ChannelBuffer>>,
    token: Option<String>,
    error_message: Option<String>,
    active_channel: Option<String>,
    _connection_task: Option<Task<()>>,
}

struct GlobalSlackStore(Entity<SlackStore>);

impl Global for GlobalSlackStore {}

impl SlackStore {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalSlackStore>().0.clone()
    }

    pub fn init(cx: &mut App) {
        let store = cx.new(|cx| Self::new(cx));
        cx.set_global(GlobalSlackStore(store.clone()));

        cx.observe_new(|channel_view: &mut ChannelView, _, cx| {
            let buffer = channel_view.channel_buffer();
            SlackStore::global(cx).update(cx, |store, cx| {
                if store.connection_state == ConnectionState::Connected {
                    store.hook_channel_buffer(buffer, cx);
                }
            });
        }).detach();
    }

    pub fn new(cx: &mut Context<Self>) -> Self {
        let credentials_task = cx.read_credentials("https://zed.dev/slack");
        cx.spawn(async move |this, cx| {
            if let Ok(Some((_, password))) = credentials_task.await {
                if let Ok(token) = String::from_utf8(password) {
                    let _ = this.update(cx, |store, cx| {
                        store.token = Some(token);
                        cx.notify();
                    });
                }
            }
        }).detach();

        Self {
            messages: Vec::new(),
            connection_state: ConnectionState::Disconnected,
            workspace: None,
            channel_buffer_subscription: None,
            channel_buffer: None,
            token: None,
            error_message: None,
            active_channel: None,
            _connection_task: None,
        }
    }

    pub fn token(&self) -> Option<String> {
        self.token.clone()
    }

    pub fn hook_channel_buffer(&mut self, _buffer: Entity<channel::ChannelBuffer>, _cx: &mut Context<Self>) {
        // Disabled ChannelBuffer hook because we are using AgentPanel hook now!
    }

    pub fn send_message(&mut self, text: String, cx: &mut Context<Self>) {
        let text = crate::markdown::format_for_slack(&text);
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return,
        };

        // Optimistically add to UI
        let msg = SlackMessage {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
            user: "You".to_string(),
            text: text.clone(),
            is_incoming: false,
            timestamp: "Just now".to_string(),
            channel: self.active_channel.clone(),
        };
        self.messages.push(msg);
        cx.notify();

        let active_channel = self.active_channel.clone().unwrap_or_else(|| "#general".to_string());
        
        cx.spawn(async move |this, mut cx| {
            let req_body = serde_json::json!({
                "channel": active_channel,
                "text": text
            });

            // Run the actual network request on the Tokio background thread
            let req_task = cx.update(|cx| {
                gpui_tokio::Tokio::spawn(cx, async move {
                    let client = reqwest::Client::new();
                    client.post("https://zed-slack-backend.wittygrass-2327b171.eastus2.azurecontainerapps.io/api/send_message")
                        .header("Authorization", format!("Bearer {}", token))
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&req_body).unwrap())
                        .send()
                        .await
                        .map(|res| (res.status(), futures::executor::block_on(res.text())))
                })
            });
            
            // Wait for the Tokio task to finish, handling it on the main GPUI thread
            let result = req_task.await;

            match result {
                Ok(Ok((status, text_res))) => {
                    if !status.is_success() {
                        let err_text = text_res.unwrap_or_else(|_| "Unknown error".to_string());
                        if let Some(this) = this.upgrade() {
                            let _ = cx.update(|cx| {
                                this.update(cx, |this, cx| {
                                    this.error_message = Some(format!("Slack API Error: {}", err_text));
                                    cx.notify();
                                })
                            });
                        }
                    }
                },
                Ok(Err(e)) => {
                    if let Some(this) = this.upgrade() {
                        let _ = cx.update(|cx| {
                            this.update(cx, |this, cx| {
                                this.error_message = Some(format!("Network Error: {}", e));
                                cx.notify();
                            })
                        });
                    }
                },
                Err(e) => {
                    // Task panicked or was cancelled
                    if let Some(this) = this.upgrade() {
                        let _ = cx.update(|cx| {
                            this.update(cx, |this, cx| {
                                this.error_message = Some(format!("Task Error: {}", e));
                                cx.notify();
                            })
                        });
                    }
                }
            }
        }).detach();
    }

    pub fn messages(&self) -> &[SlackMessage] {
        &self.messages
    }

    pub fn connection_state(&self) -> &ConnectionState {
        &self.connection_state
    }

    pub fn error_message(&self) -> Option<&String> {
        self.error_message.as_ref()
    }

    pub fn connect(&mut self, workspace: WeakEntity<Workspace>, token: String, cx: &mut Context<Self>) {
        let token = token.trim().to_string();
        self.error_message = None;
        eprintln!("Initiating native Slack WebSocket connection to backend...");
        
        let token_bytes = token.as_bytes().to_vec();
        cx.write_credentials("https://zed.dev/slack", "zed-custom-slack-token", &token_bytes).detach_and_log_err(cx);

        self.connection_state = ConnectionState::Connecting;
        self.workspace = Some(workspace.clone());
        self.token = Some(token.clone());
        cx.notify();

        let url = format!("wss://zed-slack-backend.wittygrass-2327b171.eastus2.azurecontainerapps.io/ws?token={}", token);
        eprintln!("Connecting to WebSocket at {}", url);

        let workspace_weak = workspace;
        
        self._connection_task = Some(cx.spawn(async move |this, mut cx| {
            let mut backoff_secs = 1;
            loop {
                let connect_task = cx.update(|cx| {
                gpui_tokio::Tokio::spawn_result(cx, {
                    let url = url.clone();
                    async move {
                        connect_async(&url)
                            .await
                            .map_err(|e| anyhow::anyhow!(e))
                    }
                })
            });

            let (mut ws_stream, _) = match connect_task.await {
                Ok((res, resp)) => (res, resp),
                Err(e) => {
                    eprintln!("Failed to connect WebSocket: {}", e);
                    if let Some(this) = this.upgrade() {
                        cx.update(|cx| {
                            this.update(cx, |this, cx| {
                                this.connection_state = ConnectionState::Disconnected;
                                this.error_message = Some(format!("Failed to connect: {}", e));
                                cx.notify();
                            })
                        });
                    }
                    cx.background_executor().timer(std::time::Duration::from_secs(backoff_secs)).await;
                    if backoff_secs < 60 {
                        backoff_secs *= 2;
                    }
                    continue;
                }
            };
            
            // Connection successful, reset backoff
            backoff_secs = 1;
            
            eprintln!("WebSocket upgraded successfully!");

            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    this.connection_state = ConnectionState::Connected;
                    
                    if let Some(workspace) = workspace_weak.upgrade() {
                        let mut buffers = Vec::new();
                        workspace.update(cx, |workspace, cx| {
                            for channel_view in workspace.items_of_type::<ChannelView>(cx) {
                                buffers.push(channel_view.read(cx).channel_buffer());
                            }
                        });
                        for buffer in buffers {
                            this.hook_channel_buffer(buffer, cx);
                        }
                    }
                    cx.notify();
                });
            }

            let (tx, mut rx) = futures::channel::mpsc::unbounded();

            cx.update(|cx| {
                gpui_tokio::Tokio::spawn(cx, async move {
                    while let Ok(Some(msg_res)) = tokio::time::timeout(std::time::Duration::from_secs(90), ws_stream.next()).await {
                        match msg_res {
                            Ok(Message::Text(text)) => {
                                if tx.unbounded_send(text).is_err() {
                                    break;
                                }
                            }
                            Ok(_) => {
                                // Ignore Ping, Pong, Close, and Binary frames.
                                // Because we didn't split the stream, tokio-tungstenite will automatically
                                // send a Pong response when it processes a Ping here.
                            }
                            Err(_) => {
                                break; // Network error, exit loop
                            }
                        }
                    }
                }).detach();
            });

            // Slack -> Zed Relay Loop
            loop {
                if let Some(text) = rx.next().await {
                    if let Ok(slack_msg) = serde_json::from_str::<SlackMessage>(&text) {
                        if let Some(this) = this.upgrade() {
                            this.update(cx, |this, cx| {
                                if let Some(ch) = &slack_msg.channel {
                                    this.active_channel = Some(ch.clone());
                                }
                                this.messages.push(slack_msg.clone());
                                cx.emit(SlackEvent::MessageReceived(slack_msg.clone()));
                                cx.notify();
                            });
                        }
                    }
                } else {
                    // Disconnected or error
                    if let Some(this) = this.upgrade() {
                        this.update(cx, |this, cx| {
                            this.connection_state = ConnectionState::Connecting;
                            cx.notify();
                        });
                    }
                    break;
                }
            }
            // If the connection drops, apply a 1-second backoff immediately before the next attempt
            cx.background_executor().timer(std::time::Duration::from_secs(1)).await;
            }
        }));
    }
}

impl EventEmitter<SlackEvent> for SlackStore {}
