use std::sync::Arc;

use agent_client_protocol as acp;
use agent_settings::AgentSettings;
use anyhow::{anyhow, Result};
use futures::{AsyncReadExt, FutureExt};
use gpui::{App, AppContext, Task};
use http_client::{AsyncBody, HttpClientWithUrl};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::Settings;
use ui::SharedString;

use crate::{AgentTool, ToolCallEventStream, ToolPermissionDecision, decide_permission_from_settings};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ElasticSearchToolInput {
    /// The Elasticsearch query string.
    pub query: String,
}

pub struct ElasticSearchTool {
    http_client: Arc<HttpClientWithUrl>,
}

impl ElasticSearchTool {
    pub fn new(http_client: Arc<HttpClientWithUrl>) -> Self {
        Self { http_client }
    }
}

impl AgentTool for ElasticSearchTool {
    type Input = ElasticSearchToolInput;
    type Output = String;

    fn name() -> &'static str {
        "elastic_search"
    }

    fn description() -> SharedString {
        "Executes a search query against the configured Elasticsearch endpoint and returns the JSON response.".into()
    }

    fn kind() -> acp::ToolKind {
        acp::ToolKind::Fetch
    }

    fn initial_title(
        &self,
        input: Result<Self::Input, serde_json::Value>,
        _cx: &mut App,
    ) -> SharedString {
        match input {
            Ok(input) => format!("Search Elasticsearch for '{}'", input.query).into(),
            Err(_) => "Search Elasticsearch".into(),
        }
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output>> {
        let (decision, elastic_config) = {
            let settings = AgentSettings::get_global(cx);
            (
                decide_permission_from_settings(Self::name(), &input.query, settings),
                settings.elastic_search.clone()
            )
        };

        let authorize = match decision {
            ToolPermissionDecision::Allow => None,
            ToolPermissionDecision::Deny(reason) => {
                return Task::ready(Err(anyhow::anyhow!("{}", reason)));
            }
            ToolPermissionDecision::Confirm => {
                let context = crate::ToolPermissionContext {
                    tool_name: Self::name().to_string(),
                    input_value: input.query.clone(),
                };
                Some(event_stream.authorize(
                    format!("Search Elasticsearch for '{}'", input.query),
                    context,
                    cx,
                ))
            }
        };

        let Some(elastic_config) = elastic_config else {
            return Task::ready(Err(anyhow!("Elasticsearch is not configured in settings.")));
        };

        if elastic_config.endpoint_url.is_empty() {
            return Task::ready(Err(anyhow!("Elasticsearch endpoint_url is empty in settings.")));
        }

        let search_url = {
            let base = elastic_config.endpoint_url;
            if base.ends_with("/_search") {
                base
            } else {
                format!("{}/_search", base.trim_end_matches('/'))
            }
        };
        let api_key = elastic_config.api_key;
        let http_client = self.http_client.clone();

        let fetch_task = cx.background_spawn(async move {
            if let Some(authorize) = authorize {
                authorize.await?;
            }

            let body = serde_json::json!({
                "query": {
                    "query_string": {
                        "query": input.query
                    }
                }
            });

            let mut request = http_client::Request::builder()
                .method(http_client::Method::POST)
                .uri(search_url)
                .header("Content-Type", "application/json");

            if let Some(key) = api_key {
                request = request.header("Authorization", format!("ApiKey {}", key));
            }

            let req = request.body(AsyncBody::from(serde_json::to_vec(&body)?))?;
            let mut response = http_client.send(req).await.map_err(|e| anyhow!("Failed to connect to Elasticsearch: {}", e))?;
            
            let mut response_body = Vec::new();
            response.body_mut().read_to_end(&mut response_body).await?;

            if !response.status().is_success() {
                let error_text = String::from_utf8_lossy(&response_body);
                return Err(anyhow!("Elasticsearch error ({}): {}", response.status(), error_text));
            }

            let json: serde_json::Value = serde_json::from_slice(&response_body)?;
            Ok(serde_json::to_string_pretty(&json)?)
        });

        cx.foreground_executor().spawn(async move {
            let result = futures::select! {
                res = fetch_task.fuse() => res,
                _ = event_stream.cancelled_by_user().fuse() => {
                    return Err(anyhow!("cancelled by user"));
                }
            };
            result
        })
    }
}
