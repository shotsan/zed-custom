use std::sync::Arc;

use agent_client_protocol as acp;
use agent_settings::AgentSettings;
use anyhow::{Result, bail};
use futures::{AsyncReadExt as _, FutureExt as _};
use gpui::{App, AppContext as _, Task};
use http_client::{AsyncBody, HttpClient as _, HttpClientWithUrl};
use schemars::JsonSchema;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use settings::Settings;
use ui::SharedString;

use crate::{
    AgentTool, ToolCallEventStream, ToolPermissionDecision, decide_permission_from_settings,
};

/// Search the web for the given query and return the top results as text.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchToolInput {
    /// The search query to look up on the web.
    pub query: String,
}

pub struct SearchTool {
    http_client: Arc<HttpClientWithUrl>,
    tavily_api_key: Option<String>,
}

#[derive(Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

impl SearchTool {
    pub fn new(http_client: Arc<HttpClientWithUrl>, tavily_api_key: Option<String>) -> Self {
        Self {
            http_client,
            tavily_api_key,
        }
    }

    async fn tavily_search(
        http_client: Arc<HttpClientWithUrl>,
        api_key: &str,
        query: &str,
    ) -> Result<String> {
        let request_body = serde_json::to_string(&serde_json::json!({
            "query": query,
            "max_results": 10,
            "search_depth": "basic",
            "api_key": api_key,
        }))?;

        let mut response = http_client
            .post_json("https://api.tavily.com/search", AsyncBody::from(request_body))
            .await?;

        let mut body = Vec::new();
        response.body_mut().read_to_end(&mut body).await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            let text = String::from_utf8_lossy(&body);
            bail!(
                "Tavily search failed (HTTP {}): {text}",
                response.status().as_u16()
            );
        }

        let tavily_response: TavilyResponse = serde_json::from_slice(&body)?;

        if tavily_response.results.is_empty() {
            bail!("No results found for: {query}");
        }

        let mut output = String::new();
        for (index, result) in tavily_response.results.iter().enumerate() {
            output.push_str(&format!(
                "{}. [{}]({})\n",
                index + 1,
                result.title,
                result.url
            ));
            if !result.content.is_empty() {
                output.push_str(&format!("   {}\n", result.content));
            }
            output.push('\n');
        }

        Ok(output)
    }

    async fn duckduckgo_search(
        http_client: Arc<HttpClientWithUrl>,
        query: &str,
    ) -> Result<String> {
        let encoded_query = url_encode(query);
        let url = format!("https://html.duckduckgo.com/html/?q={encoded_query}");

        let mut response = http_client.get(&url, AsyncBody::default(), true).await?;

        let mut body = Vec::new();
        response.body_mut().read_to_end(&mut body).await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            let text = String::from_utf8_lossy(&body);
            bail!("Search failed (HTTP {}): {text}", response.status().as_u16());
        }

        let html = String::from_utf8_lossy(&body).to_string();
        let results = parse_results(&html);

        if results.is_empty() {
            bail!("No results found for: {query}");
        }

        let mut output = String::new();
        for (index, result) in results.iter().enumerate() {
            output.push_str(&format!("{}. [{}]({})\n", index + 1, result.title, result.url));
            if !result.snippet.is_empty() {
                output.push_str(&format!("   {}\n", result.snippet));
            }
            output.push('\n');
        }

        Ok(output)
    }
}

struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

fn parse_results(html: &str) -> Vec<SearchResult> {
    let document = Html::parse_document(html);

    let Ok(result_selector) = Selector::parse(".result") else {
        return Vec::new();
    };
    let Ok(title_selector) = Selector::parse(".result__a") else {
        return Vec::new();
    };
    let Ok(snippet_selector) = Selector::parse(".result__snippet") else {
        return Vec::new();
    };

    let mut results = Vec::new();
    for element in document.select(&result_selector) {
        let title = element
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        let url = element
            .select(&title_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .unwrap_or_default()
            .to_string();

        let snippet = element
            .select(&snippet_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResult { title, url, snippet });
        }

        if results.len() >= 10 {
            break;
        }
    }

    results
}

fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => {
                encoded.push('+');
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

impl AgentTool for SearchTool {
    type Input = SearchToolInput;
    type Output = String;

    fn name() -> &'static str {
        "search"
    }

    fn description() -> SharedString {
        "Search the web and return the top results with titles, URLs, and snippets.".into()
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
            Ok(input) => format!("Search web for '{}'", input.query).into(),
            Err(_) => "Search web".into(),
        }
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output>> {
        let settings = AgentSettings::get_global(cx);
        let decision = decide_permission_from_settings(Self::name(), &input.query, settings);

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
                    format!("Search web for '{}'", input.query),
                    context,
                    cx,
                ))
            }
        };

        let http_client = self.http_client.clone();
        let tavily_api_key = self.tavily_api_key.clone();
        let search_task = cx.background_spawn(async move {
            if let Some(authorize) = authorize {
                authorize.await?;
            }
            if let Some(api_key) = tavily_api_key {
                Self::tavily_search(http_client, &api_key, &input.query).await
            } else {
                Self::duckduckgo_search(http_client, &input.query).await
            }
        });

        cx.foreground_executor().spawn(async move {
            futures::select! {
                result = search_task.fuse() => result,
                _ = event_stream.cancelled_by_user().fuse() => {
                    bail!("Search cancelled by user");
                }
            }
        })
    }
}
