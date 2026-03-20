use anyhow::{Context as _, Result};
use cloud_llm_client::{WebSearchResponse, WebSearchResult};
use gpui::{App, Task};
use http_client::HttpClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use web_search::{WebSearchProvider, WebSearchProviderId};

pub const TAVILY_WEB_SEARCH_PROVIDER_ID: &str = "tavily";

pub struct TavilyWebSearchProvider {
    api_key: String,
    http_client: Arc<dyn HttpClient>,
}

impl TavilyWebSearchProvider {
    pub fn new(api_key: String, http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            api_key,
            http_client,
        }
    }
}

impl WebSearchProvider for TavilyWebSearchProvider {
    fn id(&self) -> WebSearchProviderId {
        WebSearchProviderId(TAVILY_WEB_SEARCH_PROVIDER_ID.into())
    }

    fn search(&self, query: String, cx: &mut App) -> Task<Result<WebSearchResponse>> {
        let api_key = self.api_key.clone();
        let http_client = self.http_client.clone();
        cx.background_spawn(async move { perform_tavily_search(http_client, api_key, query).await })
    }
}

#[derive(Serialize)]
struct TavilySearchRequest {
    query: String,
    api_key: String,
    search_depth: String,
    max_results: usize,
    include_answer: bool,
}

#[derive(Deserialize)]
struct TavilySearchResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

async fn perform_tavily_search(
    http_client: Arc<dyn HttpClient>,
    api_key: String,
    query: String,
) -> Result<WebSearchResponse> {
    let body = TavilySearchRequest {
        query,
        api_key,
        search_depth: "basic".to_string(),
        max_results: 5,
        include_answer: false,
    };

    let request = http_client::Request::builder()
        .method(http_client::Method::POST)
        .uri("https://api.tavily.com/search")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body)?.into())?;

    let mut response = http_client
        .send(request)
        .await
        .context("failed to send Tavily search request")?;

    if !response.status().is_success() {
        let mut response_body = String::new();
        futures::AsyncReadExt::read_to_string(response.body_mut(), &mut response_body).await?;
        anyhow::bail!(
            "Tavily search request failed.\nStatus: {:?}\nBody: {response_body}",
            response.status(),
        );
    }

    let mut response_body = String::new();
    futures::AsyncReadExt::read_to_string(response.body_mut(), &mut response_body).await?;
    let tavily_response: TavilySearchResponse = serde_json::from_str(&response_body)?;

    Ok(WebSearchResponse {
        results: tavily_response
            .results
            .into_iter()
            .map(|r| WebSearchResult {
                title: r.title,
                url: r.url,
                text: r.content,
            })
            .collect(),
    })
}
