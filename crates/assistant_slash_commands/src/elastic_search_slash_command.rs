use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use gpui::{App, AppContext, Task, WeakEntity, Window};
use http_client::{AsyncBody, HttpClientWithUrl};
use language::{BufferSnapshot, LspAdapterDelegate};
use settings::Settings;
use workspace::Workspace;

use assistant_slash_command::{
    ArgumentCompletion, SlashCommand, SlashCommandOutputSection, SlashCommandResult,
};

pub struct ElasticSearchSlashCommand;

impl ElasticSearchSlashCommand {
    pub async fn elastic_search(
        http_client: Arc<HttpClientWithUrl>,
        query: &str,
        endpoint_url: &str,
        api_key: Option<&str>,
    ) -> Result<String> {
        let body = serde_json::json!({
            "query": {
                "query_string": {
                    "query": query
                }
            }
        });

        let search_url = if endpoint_url.ends_with("/_search") {
            endpoint_url.to_string()
        } else {
            let base = endpoint_url.trim_end_matches('/');
            format!("{}/_search", base)
        };

        let mut request = http_client::Request::builder()
            .method(http_client::Method::POST)
            .uri(search_url)
            .header("Content-Type", "application/json");

        if let Some(key) = api_key {
            request = request.header("Authorization", format!("ApiKey {}", key));
        }

        let req = request.body(AsyncBody::from(serde_json::to_vec(&body)?))?;
        let mut response = http_client.send(req).await.map_err(|e| anyhow::anyhow!("Failed to connect to Elasticsearch: {}", e))?;
        
        let mut response_body = Vec::new();
        futures::AsyncReadExt::read_to_end(response.body_mut(), &mut response_body).await?;

        if !response.status().is_success() {
            let error_text = String::from_utf8_lossy(&response_body);
            return Err(anyhow::anyhow!("Elasticsearch error ({}): {}", response.status(), error_text));
        }

        let json: serde_json::Value = serde_json::from_slice(&response_body)?;
        Ok(format!("```json\n{}\n```", serde_json::to_string_pretty(&json)?))
    }
}

impl SlashCommand for ElasticSearchSlashCommand {
    fn name(&self) -> String {
        "elastic".into()
    }

    fn description(&self) -> String {
        "Search Elasticsearch".into()
    }

    fn menu_text(&self) -> String {
        "Search Elasticsearch".into()
    }

    fn requires_argument(&self) -> bool {
        true
    }

    fn complete_argument(
        self: Arc<Self>,
        _arguments: &[String],
        _cancel: Arc<AtomicBool>,
        _workspace: Option<WeakEntity<Workspace>>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<Result<Vec<ArgumentCompletion>>> {
        Task::ready(Ok(Vec::new()))
    }

    fn run(
        self: Arc<Self>,
        arguments: &[String],
        _context_slash_command_output_sections: &[SlashCommandOutputSection<language::Anchor>],
        _context_buffer: BufferSnapshot,
        workspace: WeakEntity<Workspace>,
        _delegate: Option<Arc<dyn LspAdapterDelegate>>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Task<SlashCommandResult> {
        let query = arguments.join(" ");
        if query.is_empty() {
            return Task::ready(Err(anyhow::anyhow!("missing search query")));
        }

        let settings = agent_settings::AgentSettings::get_global(cx);
        let Some(elastic_config) = &settings.elastic_search else {
            return Task::ready(Err(anyhow::anyhow!("Elasticsearch is not configured in settings.")));
        };

        if elastic_config.endpoint_url.is_empty() {
            return Task::ready(Err(anyhow::anyhow!("Elasticsearch endpoint_url is empty in settings.")));
        }

        let Some(workspace) = workspace.upgrade() else {
            return Task::ready(Err(anyhow::anyhow!("workspace was dropped")));
        };

        let endpoint_url = elastic_config.endpoint_url.clone();
        let api_key = elastic_config.api_key.clone();
        let http_client = workspace.read(cx).client().http_client();

        cx.background_spawn(async move {
            let text = Self::elastic_search(http_client, &query, &endpoint_url, api_key.as_deref()).await?;

            let event = assistant_slash_command::SlashCommandEvent::Content(assistant_slash_command::SlashCommandContent::Text {
                text,
                run_commands_in_text: false,
            });

            let stream = futures::stream::once(async move { Ok(event) }).boxed();
            Ok(stream)
        })
    }
}
