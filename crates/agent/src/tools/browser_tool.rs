use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use agent_client_protocol as acp;
use agent_settings::AgentSettings;
use anyhow::{Context as _, Result, anyhow, bail};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::{AsyncReadExt as _, FutureExt as _, StreamExt as _};
use gpui::{App, AppContext as _, Task};
use gpui_tokio::Tokio;
use html_to_markdown::{TagHandler, convert_html_to_markdown, markdown};
use http_client::{AsyncBody, HttpClientWithUrl};
use language_model::LanguageModelToolResultContent;
use schemars::JsonSchema;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use settings::Settings;
use ui::SharedString;
use util::markdown::{MarkdownEscaped, MarkdownInlineCode};

use crate::{
    AgentTool, ToolCallEventStream, ToolPermissionDecision, decide_permission_from_settings,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub index: usize,
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserToolOutput {
    pub action: String,
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub results: Vec<SearchResult>,
}

impl From<BrowserToolOutput> for LanguageModelToolResultContent {
    fn from(value: BrowserToolOutput) -> Self {
        let mut output = String::new();

        if !value.results.is_empty() {
            for result in &value.results {
                output.push_str(&format!(
                    "[{}] {} - {}\n    {}\n\n",
                    result.index, result.title, result.url, result.snippet
                ));
            }
        }

        if !value.content.is_empty() {
            output.push_str(&value.content);
        }

        output.into()
    }
}

/// Browse the web using a headless browser.
/// Use "search" to find pages, then "navigate" to read a specific page's content.
///
/// For search: provide a "query" string.
/// For navigate: provide a "url" string (a full URL or a result index like "1" from a previous search).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BrowserToolInput {
    /// The action to perform: "search" or "navigate".
    pub action: String,
    /// For "search": the search query. For "navigate": a URL or result index from a previous search.
    pub target: String,
}

struct ChromeSession {
    browser: Browser,
    _handler_task: tokio::task::JoinHandle<()>,
}

impl ChromeSession {
    async fn launch() -> Result<Self> {
        let user_data_dir = std::env::temp_dir().join(format!("zed-browser-tool-{}", uuid::Uuid::new_v4()));
        let config = BrowserConfig::builder()
            .no_sandbox()
            .user_data_dir(user_data_dir)
            .build()
            .map_err(|error| anyhow!("Failed to build Chrome config: {error}"))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .context("Failed to launch headless Chrome. Is Chrome or Chromium installed?")?;

        let handler_task = tokio::spawn(async move {
            while handler.next().await.is_some() {}
        });

        Ok(Self {
            browser,
            _handler_task: handler_task,
        })
    }

    async fn render_page(&self, url: &str) -> Result<String> {
        let page = self
            .browser
            .new_page(url)
            .await
            .context("Failed to open page in Chrome")?;

        let html = page
            .wait_for_navigation()
            .await
            .context("Navigation failed")?
            .content()
            .await
            .context("Failed to extract page content")?;

        Self::close_page(page).await;

        Ok(html)
    }

    async fn close_page(page: Page) {
        let _ = page.close().await;
    }
}

impl Drop for ChromeSession {
    fn drop(&mut self) {
        self._handler_task.abort();
    }
}

pub struct BrowserTool {
    http_client: Arc<HttpClientWithUrl>,
    chrome_session: Arc<tokio::sync::Mutex<Option<ChromeSession>>>,
}

impl BrowserTool {
    pub fn new(http_client: Arc<HttpClientWithUrl>) -> Self {
        Self {
            http_client,
            chrome_session: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    async fn get_or_launch_chrome(
        session: &tokio::sync::Mutex<Option<ChromeSession>>,
    ) -> Result<()> {
        let mut guard = session.lock().await;
        if guard.is_none() {
            *guard = Some(ChromeSession::launch().await?);
        }
        Ok(())
    }

    async fn render_with_chrome(
        session: &tokio::sync::Mutex<Option<ChromeSession>>,
        url: &str,
    ) -> Result<String> {
        let guard = session.lock().await;
        let chrome = guard
            .as_ref()
            .context("Chrome session not initialized")?;
        chrome.render_page(url).await
    }

    pub fn parse_search_results(html_body: &str) -> Vec<SearchResult> {
        let document = Html::parse_document(html_body);

        let result_selector = match Selector::parse(".result") {
            Ok(selector) => selector,
            Err(_) => return Vec::new(),
        };
        let title_selector = match Selector::parse(".result__a") {
            Ok(selector) => selector,
            Err(_) => return Vec::new(),
        };
        let snippet_selector = match Selector::parse(".result__snippet") {
            Ok(selector) => selector,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();
        for (index, element) in document.select(&result_selector).enumerate() {
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
                results.push(SearchResult {
                    index: index + 1,
                    title,
                    url,
                    snippet,
                });
            }

            if results.len() >= 10 {
                break;
            }
        }

        results
    }

    fn html_to_clean_markdown(html_body: &[u8], url: &str) -> Result<String> {
        let mut handlers: Vec<TagHandler> = vec![
            Rc::new(RefCell::new(markdown::WebpageChromeRemover)),
            Rc::new(RefCell::new(markdown::ParagraphHandler)),
            Rc::new(RefCell::new(markdown::HeadingHandler)),
            Rc::new(RefCell::new(markdown::ListHandler)),
            Rc::new(RefCell::new(markdown::TableHandler::new())),
            Rc::new(RefCell::new(markdown::StyledTextHandler)),
        ];

        if url.contains("wikipedia.org") {
            use html_to_markdown::structure::wikipedia;
            handlers.push(Rc::new(RefCell::new(wikipedia::WikipediaChromeRemover)));
            handlers.push(Rc::new(RefCell::new(wikipedia::WikipediaInfoboxHandler)));
            handlers.push(Rc::new(RefCell::new(
                wikipedia::WikipediaCodeHandler::new(),
            )));
        } else {
            handlers.push(Rc::new(RefCell::new(markdown::CodeHandler)));
        }

        convert_html_to_markdown(html_body, &mut handlers)
    }

    pub async fn fetch_raw_html(
        http_client: Arc<HttpClientWithUrl>,
        url: &str,
    ) -> Result<(Vec<u8>, String)> {
        let final_url = if !url.starts_with("https://") && !url.starts_with("http://") {
            format!("https://{url}")
        } else {
            url.to_string()
        };

        let mut response = http_client
            .get(&final_url, AsyncBody::default(), true)
            .await?;

        let mut body = Vec::new();
        response
            .body_mut()
            .read_to_end(&mut body)
            .await
            .context("error reading response body")?;

        if response.status().is_client_error() || response.status().is_server_error() {
            let text = String::from_utf8_lossy(&body);
            bail!("HTTP error {}: {}", response.status().as_u16(), text);
        }

        Ok((body, final_url))
    }
}

impl AgentTool for BrowserTool {
    type Input = BrowserToolInput;
    type Output = BrowserToolOutput;

    fn name() -> &'static str {
        "browser"
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
            Ok(input) => match input.action.as_str() {
                "search" => format!("Searching: {}", MarkdownEscaped(&input.target)).into(),
                "navigate" => format!("Reading: {}", MarkdownEscaped(&input.target)).into(),
                _ => "Browsing".into(),
            },
            Err(_) => "Browsing".into(),
        }
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output>> {
        let settings = AgentSettings::get_global(cx);
        let decision = decide_permission_from_settings(Self::name(), &input.target, settings);

        let authorize = match decision {
            ToolPermissionDecision::Allow => None,
            ToolPermissionDecision::Deny(reason) => {
                return Task::ready(Err(anyhow!("{}", reason)));
            }
            ToolPermissionDecision::Confirm => {
                let description = match input.action.as_str() {
                    "search" => format!(
                        "Search the web for {}",
                        MarkdownInlineCode(&input.target)
                    ),
                    "navigate" => format!(
                        "Navigate to {}",
                        MarkdownInlineCode(&input.target)
                    ),
                    other => format!("Browser action: {other}"),
                };
                let context = crate::ToolPermissionContext {
                    tool_name: "browser".to_string(),
                    input_value: input.target.clone(),
                };
                Some(event_stream.authorize(description, context, cx))
            }
        };

        let http_client = self.http_client.clone();
        let chrome_session = self.chrome_session.clone();
        let action = input.action;
        let target = input.target;

        let chrome_task = Tokio::spawn_result(cx, async move {
                if let Some(authorize) = authorize {
                    authorize.await?;
                }

                match action.as_str() {
                    "search" => {
                        let search_url = format!(
                            "https://html.duckduckgo.com/html/?q={}",
                            urlencoding(&target)
                        );
                        // Try Chrome rendering first, fall back to HTTP
                        let html = match Self::get_or_launch_chrome(&chrome_session).await {
                            Ok(()) => {
                                Self::render_with_chrome(&chrome_session, &search_url)
                                    .await
                                    .ok()
                            }
                            Err(_) => None,
                        };

                        Ok(FetchedContent {
                            action: "search".to_string(),
                            target,
                            html,
                            final_url: search_url,
                            needs_http_fallback: true,
                        })
                    }
                    "navigate" => {
                        let final_url =
                            if !target.starts_with("https://") && !target.starts_with("http://") {
                                format!("https://{target}")
                            } else {
                                target.clone()
                            };

                        let html = match Self::get_or_launch_chrome(&chrome_session).await {
                            Ok(()) => {
                                Self::render_with_chrome(&chrome_session, &final_url)
                                    .await
                                    .ok()
                            }
                            Err(_) => None,
                        };

                        Ok(FetchedContent {
                            action: "navigate".to_string(),
                            target,
                            html,
                            final_url,
                            needs_http_fallback: true,
                        })
                    }
                    other => {
                        bail!("Unknown browser action: {other}. Use 'search' or 'navigate'.")
                    }
                }
            });

        let fetch_task = cx.background_spawn({
            let http_client = http_client;
            let event_stream = event_stream.clone();
            async move {
                let fetched = chrome_task.await?;

                let (body, final_url) = if let Some(html) = fetched.html {
                    (html.into_bytes(), fetched.final_url)
                } else if fetched.needs_http_fallback {
                    let (body, url) =
                        Self::fetch_raw_html(http_client, &fetched.final_url).await?;
                    (body, url)
                } else {
                    bail!("Failed to fetch page content");
                };

                Ok((fetched.action, fetched.target, body, final_url, event_stream))
            }
        });

        cx.foreground_executor().spawn(async move {
            let (action, target, body, final_url, event_stream) = futures::select! {
                result = fetch_task.fuse() => result?,
                _ = event_stream.cancelled_by_user().fuse() => {
                    bail!("Browser action cancelled by user");
                }
            };

            match action.as_str() {
                "search" => {
                    let html_string = String::from_utf8_lossy(&body).to_string();
                    let results = Self::parse_search_results(&html_string);

                    if results.is_empty() {
                        bail!("No search results found for: {target}");
                    }

                    let result_count = results.len();
                    event_stream.update_fields(
                        acp::ToolCallUpdateFields::new()
                            .title(format!("Found {result_count} results"))
                            .content(
                                results
                                    .iter()
                                    .map(|result| {
                                        acp::ToolCallContent::Content(acp::Content::new(
                                            acp::ContentBlock::ResourceLink(
                                                acp::ResourceLink::new(
                                                    result.title.clone(),
                                                    result.url.clone(),
                                                )
                                                .description(result.snippet.clone()),
                                            ),
                                        ))
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                    );

                    Ok(BrowserToolOutput {
                        action: "search".to_string(),
                        content: String::new(),
                        results,
                    })
                }
                "navigate" => {
                    let content = Self::html_to_clean_markdown(&body, &final_url)?;

                    if content.trim().is_empty() {
                        bail!("No textual content found at: {target}");
                    }

                    event_stream.update_fields(
                        acp::ToolCallUpdateFields::new()
                            .title(format!("Read {}", MarkdownEscaped(&final_url))),
                    );

                    Ok(BrowserToolOutput {
                        action: "navigate".to_string(),
                        content,
                        results: Vec::new(),
                    })
                }
                other => {
                    bail!("Unknown browser action: {other}. Use 'search' or 'navigate'.");
                }
            }
        })
    }
}

struct FetchedContent {
    action: String,
    target: String,
    html: Option<String>,
    final_url: String,
    needs_http_fallback: bool,
}

fn urlencoding(input: &str) -> String {
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
