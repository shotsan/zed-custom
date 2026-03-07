use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use anyhow::{Result, bail};
use assistant_slash_command::{
    ArgumentCompletion, SlashCommand, SlashCommandOutput, SlashCommandOutputSection,
    SlashCommandResult,
};
use futures::AsyncReadExt;
use gpui::{Task, WeakEntity};
use http_client::{AsyncBody, HttpClientWithUrl};
use language::{BufferSnapshot, LspAdapterDelegate};
use scraper::{Html, Selector};
use ui::prelude::*;
use workspace::Workspace;

#[derive(Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

pub struct SearchSlashCommand;

impl SearchSlashCommand {
    pub async fn search(
        http_client: Arc<HttpClientWithUrl>,
        query: &str,
    ) -> Result<Vec<SearchResult>> {
        let encoded_query = urlencoding(query);
        let url = format!("https://html.duckduckgo.com/html/?q={encoded_query}");

        let mut response = http_client.get(&url, AsyncBody::default(), true).await?;

        let mut body = Vec::new();
        response
            .body_mut()
            .read_to_end(&mut body)
            .await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            let text = String::from_utf8_lossy(&body);
            bail!("Search failed (HTTP {}): {text}", response.status().as_u16());
        }

        let html_string = String::from_utf8_lossy(&body).to_string();
        Ok(parse_search_results(&html_string))
    }
}

fn parse_search_results(html_body: &str) -> Vec<SearchResult> {
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
            results.push(SearchResult {
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

impl SlashCommand for SearchSlashCommand {
    fn name(&self) -> String {
        "search".into()
    }

    fn description(&self) -> String {
        "Search the web and insert results".into()
    }

    fn icon(&self) -> IconName {
        IconName::MagnifyingGlass
    }

    fn menu_text(&self) -> String {
        self.description()
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
        _: &mut Window,
        cx: &mut App,
    ) -> Task<SlashCommandResult> {
        let query = arguments.join(" ");
        if query.trim().is_empty() {
            return Task::ready(Err(anyhow::anyhow!("Please provide a search query")));
        }

        let Some(workspace) = workspace.upgrade() else {
            return Task::ready(Err(anyhow::anyhow!("workspace was dropped")));
        };

        let http_client = workspace.read(cx).client().http_client();
        let query_display = query.clone();

        let search_task = cx.background_spawn(async move {
            Self::search(http_client, &query).await
        });

        cx.foreground_executor().spawn(async move {
            let results = search_task.await?;

            if results.is_empty() {
                bail!("No results found for: {query_display}");
            }

            let mut text = String::new();
            for (index, result) in results.iter().enumerate() {
                text.push_str(&format!(
                    "{}. [{}]({})\n",
                    index + 1,
                    result.title,
                    result.url
                ));
                if !result.snippet.is_empty() {
                    text.push_str(&format!("   {}\n", result.snippet));
                }
                text.push('\n');
            }

            let range = 0..text.len();
            Ok(SlashCommandOutput {
                text,
                sections: vec![SlashCommandOutputSection {
                    range,
                    icon: IconName::MagnifyingGlass,
                    label: format!("search: {query_display}").into(),
                    metadata: None,
                }],
                run_commands_in_text: false,
            }
            .into_event_stream())
        })
    }
}
