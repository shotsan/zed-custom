use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use anyhow::{Result, bail};
use gpui::{App, Task, SharedString};
use schemars::JsonSchema;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use http_client::{AsyncBody, HttpClientWithUrl};
use chromiumoxide::{Browser, BrowserConfig};
use html_to_markdown::{TagHandler, convert_html_to_markdown, markdown};

use crate::{AgentTool, ToolCallEventStream};
use agent_client_protocol as acp;
use agent_settings::AgentSettings;
use language_model::LanguageModelToolResultContent;
use util::markdown::MarkdownEscaped;
use settings::Settings;
use futures::{AsyncReadExt, StreamExt as _};
use std::fmt::Write as _;
use http_client::HttpRequestExt as _;


/// Perform a deep, multi-tab graph-based research dive on a complex topic.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeepResearchToolInput {
    /// The deep research complex topic to investigate.
    pub topic: String,
    /// An optional list of specific domains to restrict the research to (e.g., ["github.com", "docs.rs"]).
    pub domains: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepResearchToolOutput {
    pub report: String,
}

impl From<DeepResearchToolOutput> for LanguageModelToolResultContent {
    fn from(value: DeepResearchToolOutput) -> Self {
        value.report.into()
    }
}

pub struct DeepResearchTool {
    http_client: Arc<HttpClientWithUrl>,
}

impl DeepResearchTool {
    pub fn new(http_client: Arc<HttpClientWithUrl>) -> Self {
        Self { http_client }
    }
}

impl AgentTool for DeepResearchTool {
    type Input = DeepResearchToolInput;
    type Output = DeepResearchToolOutput;

    fn name() -> &'static str {
        "deep_research"
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
            Ok(input) => format!("Deep Researching: {}", MarkdownEscaped(&input.topic)).into(),
            Err(_) => "Deep Researching...".into(),
        }
    }

    fn run(
        self: Arc<Self>,
        input: Self::Input,
        event_stream: ToolCallEventStream,
        cx: &mut App,
    ) -> Task<Result<Self::Output>> {
        let settings = AgentSettings::get_global(cx).deep_research.clone();
        let http_client = self.http_client.clone();
        
        // 1. Prepare Expansion Prompt
        // 1. Get the current active model from the LanguageModelRegistry


        let model = language_model::LanguageModelRegistry::read_global(cx)
            .default_model()
            .map(|m| m.model);
        let mut async_cx = cx.to_async();
        let tokio_handle = gpui_tokio::Tokio::handle(cx);
        cx.foreground_executor().spawn(async move {
            event_stream.update_fields(acp::ToolCallUpdateFields::new().title("Expanding topic into search queries..."));
            let queries = expand_topic(
                http_client.clone(),
                &input.topic,
                input.domains.as_ref(),
                settings.search_system_prompt.as_ref().map(|s| s.as_ref()),
                model.clone(),
                &mut async_cx
            ).await.unwrap_or_else(|_| vec![input.topic.clone()]);
            
            let event_stream_clone = event_stream.clone();
            let model_clone = model.clone();
            let mut async_cx_clone = async_cx.clone();
            let bg_task = async_cx.foreground_executor().spawn({
                let http_client = http_client.clone();
                let topic = input.topic.clone();
                let queries = queries.clone();
                let domains = input.domains.clone();
                let max_concurrent_tabs = settings.max_concurrent_tabs;
                let max_iterations = settings.max_depth;
                let event_stream = event_stream_clone;
                let tokio_handle = tokio_handle.clone();
                let gap_analysis_prompt = settings.gap_analysis_system_prompt.as_ref().map(|s| s.to_string());
                async move {
                    run_deep_research_bg(http_client, topic, queries, domains, max_concurrent_tabs, max_iterations, Some(event_stream), model_clone, &mut async_cx_clone, tokio_handle, gap_analysis_prompt).await
                }
            });


            
            match bg_task.await {
                Ok(raw_report) => {
                    let summary = condense_report(
                        http_client,
                        &input.topic,
                        &raw_report,
                        settings.condensation_system_prompt.as_ref().map(|s| s.as_ref()),
                        model,
                        &mut async_cx
                    ).await.unwrap_or_else(|_| raw_report);
                    Ok(DeepResearchToolOutput { report: summary })
                }
                Err(e) => Ok(DeepResearchToolOutput { report: format!("Failed deep research: {}", e) }),
            }
        })
    }
}

pub async fn expand_topic(
    _http_client: Arc<HttpClientWithUrl>,
    topic: &str,
    domains: Option<&Vec<String>>,
    custom_prompt: Option<&str>,
    model: Option<Arc<dyn language_model::LanguageModel>>,
    async_cx: &mut gpui::AsyncApp,
) -> Result<Vec<String>> {
    let mut queries = vec![topic.to_string()];
    
    if let Some(model) = model {
        let system_prompt = custom_prompt.unwrap_or(
            "You are a world-class investigative research analyst. Your task is to expand the provided topic into 6 highly specific and diverse search queries.
            Ensure you target a broad spectrum of data sources:
            1. Official Company Statements (Investor Relations, Annual Reports, SEC filings).
            2. Technical Specifications or GitHub repositories.
            3. Financial and Market Analysis (diverse providers).
            4. Recent Press Releases and News.
            5. Expert critiques or technical white papers.
            
            Avoid duplicate queries. Each query must target a distinct 'angle' of the topic.
            Provide ONLY the queries, one per line, with no extra text or formatting."
        );

        let mut user_message = format!("Expand this topic into 6 diverse search queries: '{}'.", topic);
        if let Some(d) = domains {
            user_message.push_str(&format!(" Heavily prioritize the following domains: {}.", d.join(", ")));
        }
        user_message.push_str("\nReturn ONLY the 6 distinct queries, one per line.");

        let request = language_model::LanguageModelRequest {
            messages: vec![language_model::LanguageModelRequestMessage {
                role: language_model::Role::System,
                content: vec![system_prompt.into()],
                cache: false,
                reasoning_details: None,
            }, language_model::LanguageModelRequestMessage {
                role: language_model::Role::User,
                content: vec![user_message.into()],
                cache: false,
                reasoning_details: None,
            }],
            ..Default::default()
        };
        
        if let Ok(mut stream) = model.stream_completion_text(request, async_cx).await {
            let mut response_text = String::new();
            use futures::StreamExt as _;
            while let Some(chunk) = stream.stream.next().await {
                if let Ok(text) = chunk {
                    response_text.push_str(text.as_ref());
                }
            }
            
            for line in response_text.lines() {
                let clean_line = line.trim().trim_start_matches("- ").trim_start_matches("* ").trim_start_matches(|c: char| c.is_ascii_digit() || c == '.').trim().to_string();
                if !clean_line.is_empty() {
                    queries.push(clean_line);
                }
            }
        }
    }
    
    queries.dedup();
    Ok(queries)
}


pub async fn run_deep_research_bg(
    http_client: Arc<HttpClientWithUrl>,
    topic: String,
    queries: Vec<String>,
    domains: Option<Vec<String>>,
    max_concurrent_tabs: usize,
    max_iterations: usize,
    event_stream: Option<ToolCallEventStream>,
    model: Option<Arc<dyn language_model::LanguageModel>>,
    async_cx: &mut gpui::AsyncApp,
    tokio_handle: tokio::runtime::Handle,
    gap_analysis_custom_prompt: Option<String>,
) -> anyhow::Result<String> {

    let mut logs = Vec::new();
    let mut log_message = |message: String, title: Option<String>| {
        logs.push(message.clone());
        if let Some(event_stream) = &event_stream {
            let mut fields = acp::ToolCallUpdateFields::new().content(
                logs.iter()
                    .map(|l: &String| acp::ToolCallContent::from(l.clone()))
                    .collect::<Vec<acp::ToolCallContent>>()
            );
            if let Some(t) = title {
                fields = fields.title(t);
            }
            event_stream.update_fields(fields);
        }
    };


    log_message("Searching DuckDuckGo for candidates...".to_string(), None);

    // 1. Fetch raw candidates
    let mut search_futures = Vec::new();
    for q in &queries {
        let http_client = http_client.clone();
        let q_clone = q.clone();
        search_futures.push(async move {
            ddg_search(http_client, &q_clone).await.unwrap_or_default()
        });
    }
    
    log::info!("🔍 Deep Research: Executing {} search queries...", queries.len());
    let mut results = Vec::new();
    for r in futures::future::join_all(search_futures).await {
        results.extend(r);
    }
    log::info!("✅ Deep Research: Found {} raw candidate sources.", results.len());
    
    if results.is_empty() {
        log_message(format!("⚠️ No search results found for: {}", topic), Some("Search Limited".to_string()));
    }
    
    log_message(format!("🔍 Found {} candidate sources. Ranking...", results.len()), None);


    // 2. Deduplicate URLs (Now that they are unmasked)
    let mut unique_urls = std::collections::HashSet::new();
    results.retain(|r| {
        let clean_url = r.url.to_lowercase().split('?').next().unwrap_or(&r.url).trim_end_matches('/').to_string();
        unique_urls.insert(clean_url)
    });
    log::info!("✅ Deep Research: Deduplicated to {} unique high-fidelity sources.", results.len());

    // 3. Rank Results using LLM intelligence
    if let Some(ref model) = model {
        let mut async_cx_clone = async_cx.clone();
        let topic_clone = topic.clone();
        if let Err(e) = rank_results_with_llm(&topic_clone, &mut results, model, &mut async_cx_clone).await {
            log::warn!("LLM Ranking failed, falling back to heuristic: {}", e);
            score_results_heuristic(&mut results, &topic_clone, domains.as_deref());
        }
    } else {
        score_results_heuristic(&mut results, &topic, domains.as_deref());
    }
    
    let _original_results = results.clone();
    
    // 5. Iterative Fetch & Discovery Loop
    let mut research_pool = results.clone();
    let mut successful_results = Vec::new();
    let mut status_entries = Vec::new();
    
    struct StatusEntry {
        status: String,
        idx: usize,
        title: String,
        url: String,
    }

    let mut iteration = 1;
    
    while iteration <= max_iterations && successful_results.len() < max_concurrent_tabs && !research_pool.is_empty() {
        log_message(format!("🚀 Iteration {}: Fetching and analyzing {} priority sources...", iteration, (max_concurrent_tabs - successful_results.len()).min(research_pool.len())), None);
        log::info!("🚀 Deep Research: Iteration {} starting fetch...", iteration);
        
        let mut candidates_to_fetch = Vec::new();
        while candidates_to_fetch.len() < (max_concurrent_tabs - successful_results.len()) && !research_pool.is_empty() {
            candidates_to_fetch.push(research_pool.remove(0));
        }
        
        if candidates_to_fetch.is_empty() { break; }
        
        let urls_to_fetch: Vec<String> = candidates_to_fetch.iter().map(|c| c.url.clone()).collect();
        for url in &urls_to_fetch {
             log::info!("🌐 Deep Research: Attempting to fetch: {}", url);
        }

        let summaries = {
             let http_client = http_client.clone();
             let event_stream = event_stream.clone();
             tokio_handle.spawn(async move {
                 browse_parallel(http_client, urls_to_fetch, event_stream).await
             }).await??
        };
        
        for (res, summary) in candidates_to_fetch.into_iter().zip(summaries.into_iter()) {
            let is_failure = summary.starts_with('(');
            if is_failure {
                log::warn!("❌ Deep Research: Blocked or Failed: {} ({})", res.url, summary);
                log_message(format!("❌ {} — {}", res.title, res.url), None);
                status_entries.push(StatusEntry {
                    status: format!("❌ Failed: {}", summary),
                    idx: status_entries.len() + 1,
                    title: res.title,
                    url: res.url,
                });
            } else {
                log::info!("✅ Deep Research: Successfully analyzed: {}", res.url);
                log_message(format!("✅ {} — {}", res.title, res.url), None);
                status_entries.push(StatusEntry {
                    status: "✅ Analyzed".to_string(),
                    idx: status_entries.len() + 1,
                    title: res.title.clone(),
                    url: res.url.clone(),
                });
                successful_results.push((res, summary));
            }
        }
        
        // Run gap analysis after every iteration except the last, as long as there are
        // still unfilled slots. This makes max_depth > 2 genuinely discover new sources
        // rather than just exhausting the existing pool.
        if iteration < max_iterations && successful_results.len() < max_concurrent_tabs {
            log_message("🧠 Identifying information gaps for targeted follow-up...".to_string(), None);
            if let Some(model) = model.as_ref() {
                let mut collected_text = String::new();
                for (_, s) in &successful_results {
                    collected_text.push_str(s);
                    collected_text.push_str("\n---\n");
                }
                
                let follow_up_queries = identify_gaps_and_relaunch(
                    &topic,
                    &collected_text,
                    model,
                    async_cx,
                    gap_analysis_custom_prompt.as_deref(),
                ).await.unwrap_or_default();
                
                if !follow_up_queries.is_empty() {
                    log_message(format!("🔍 Launching {} targeted follow-up searches for missing data...", follow_up_queries.len()), None);
                    let mut follow_up_futures = Vec::new();
                    for q in follow_up_queries {
                        let http_client = http_client.clone();
                        follow_up_futures.push(async move {
                            ddg_search(http_client, &q).await.unwrap_or_default()
                        });
                    }
                    
                    let new_results = futures::future::join_all(follow_up_futures).await;
                    for r_set in new_results {
                        for mut r in r_set {
                            // Check if URL already in pool or entries
                            let clean_url = r.url.split('?').next().unwrap_or(&r.url).trim_end_matches('/').to_string();
                            if !unique_urls.contains(&clean_url) {
                                unique_urls.insert(clean_url);
                                r.score = 50; // Boost new targeted results
                                research_pool.push(r);
                            }
                        }
                    }
                    
                    // Re-rank pool with new additions
                    let _ = rank_results_with_llm(&topic, &mut research_pool, model, async_cx).await;
                }
            }
        }
        
        iteration += 1;
    }

    log_message("✅ Data collection complete. Finalizing report...".to_string(), Some("Synthesizing...".to_string()));

    let mut raw_report = format!("# Deep Research Discovery Report: {}\n\n", topic);
    raw_report.push_str("## Investigative Status Table\n\n");
    raw_report.push_str("| Status | # | Title | URL |\n");
    raw_report.push_str("| :--- | :--- | :--- | :--- |\n");
    
    for entry in &status_entries {
        let _ = writeln!(raw_report, "| {} | {} | {} | [{}]({}) |", entry.status, entry.idx, entry.title.replace('|', "\\|"), entry.url, entry.url);
    }
    
    // Also include those we never even tried
    for (idx, r) in research_pool.iter().enumerate().take(10) {
        let _ = writeln!(raw_report, "| ⏸️ Unreached | {} | {} | [{}]({}) |", status_entries.len() + idx + 1, r.title.replace('|', "\\|"), r.url, r.url);
    }
    
    raw_report.push_str("\n---\n\n");

    for (res, page_markdown) in &successful_results {
        raw_report.push_str(&format!("## Source: {}\nSource URL: {}\n\n{}\n\n---\n", res.title, res.url, page_markdown.chars().take(12000).collect::<String>()));
    }
    
    Ok(raw_report)
}

pub async fn identify_gaps_and_relaunch(
    topic: &str,
    collected_content: &str,
    model: &Arc<dyn language_model::LanguageModel>,
    async_cx: &mut gpui::AsyncApp,
    custom_prompt: Option<&str>,
) -> Result<Vec<String>> {
    let prompt = custom_prompt.map(|p| p.to_string()).unwrap_or_else(|| format!(
        "You are a world-class investigative researcher. You have been researching '{}'.\n\
        Here is the content you have collected so far:\n\n\
        [START COLLECTED CONTENT]\n\
        {}\n\
        [END COLLECTED CONTENT]\n\n\
        Identify 3 critical information gaps or missing specific data points (e.g., hard numbers, specific projections, competitive nuances) that were NOT found in the content above.\n\
        Generate 3 highly specific search queries to find this missing information.\n\
        Return ONLY the 3 queries, one per line.",
        topic,
        if collected_content.len() > 10000 { &collected_content[..10000] } else { collected_content }
    ));

    let request = language_model::LanguageModelRequest {
        messages: vec![language_model::LanguageModelRequestMessage {
            role: language_model::Role::User,
            content: vec![prompt.into()],
            cache: false,
            reasoning_details: None,
        }],
        ..Default::default()
    };

    let mut response_text = String::new();
    if let Ok(mut stream) = model.stream_completion_text(request, async_cx).await {
        while let Some(chunk) = stream.stream.next().await {
            if let Ok(text) = chunk {
                response_text.push_str(&text);
            }
        }
    }

    let mut queries = Vec::new();
    for line in response_text.lines() {
        let clean = line.trim().trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '-' || c == '*').trim().to_string();
        if !clean.is_empty() {
            queries.push(clean);
        }
    }
    Ok(queries)
}

pub async fn condense_report(
    _http_client: Arc<HttpClientWithUrl>,
    topic: &str,
    raw_report: &str,
    custom_prompt: Option<&str>,
    model: Option<Arc<dyn language_model::LanguageModel>>,
    async_cx: &mut gpui::AsyncApp,
) -> Result<String> {
    if let Some(model) = model {
        let system_prompt = custom_prompt.unwrap_or(
            "You are an expert technical researcher synthesizing deep research data.\n\
            Analyze the raw research material from MULTIPLE sources and provide a highly detailed, coherent, and comprehensive Markdown report. \
            Citing multiple diverse sources is CRITICAL. Do not rely on just one primary source if others are available. \
            Synthesize cross-source data points to provide the most authoritative view."
        );

        let user_message = format!(
            "The user asked to deeply research: '{}'.\n\
            Below is the raw, unedited text scraped from top ranking web pages related to the topic.\n\
            \n\
            ### Raw Research Material\n\
            {}\n\
            \n\
            Provide a technical, high-density report.",
            topic, raw_report
        );

        let request = language_model::LanguageModelRequest {
            messages: vec![language_model::LanguageModelRequestMessage {
                role: language_model::Role::System,
                content: vec![system_prompt.into()],
                cache: false,
                reasoning_details: None,
            }, language_model::LanguageModelRequestMessage {
                role: language_model::Role::User,
                content: vec![user_message.into()],
                cache: false,
                reasoning_details: None,
            }],
            ..Default::default()
        };

        if let Ok(mut stream) = model.stream_completion_text(request, async_cx).await {
            let mut response_text = String::new();
            use futures::StreamExt as _;
            while let Some(chunk) = stream.stream.next().await {
                if let Ok(text) = chunk {
                    response_text.push_str(text.as_ref());
                }
            }
            if !response_text.is_empty() {
                return Ok(response_text);
            }
        }
    }
    
    // Fall back to just returning the raw report if LLM fails
    Ok(format!("LLM Condensation Failed. Showing Raw Report:\n\n{}", raw_report))
}


#[derive(Clone, Serialize, Deserialize)]
pub struct DeepResearchSearchResult {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub score: i32,
}

async fn ddg_search(http_client: Arc<HttpClientWithUrl>, query: &str) -> Result<Vec<DeepResearchSearchResult>> {
    let encoded_query = url_encode(query);
    let url = format!("https://html.duckduckgo.com/html/?q={encoded_query}");

    let mut response = http_client.get(&url, AsyncBody::default(), true).await?;

    let mut body = Vec::new();
    response.body_mut().read_to_end(&mut body).await?;

    if response.status().is_client_error() || response.status().is_server_error() {
        bail!("Search failed with status: {}", response.status());
    }

    let html = String::from_utf8_lossy(&body);
    Ok(parse_results(&html))
}

fn score_results_heuristic(results: &mut [DeepResearchSearchResult], topic: &str, domains: Option<&[String]>) {
    let query_words: Vec<String> = topic.to_lowercase().split_whitespace().map(|s| s.to_string()).collect();

    for result in results.iter_mut() {
        let mut score = 0;
        let snippet_lower = result.snippet.to_lowercase();
        let title_lower = result.title.to_lowercase();
        let url_lower = result.url.to_lowercase();

        // Keyword Overlap (+2 points per word)
        for word in &query_words {
            if snippet_lower.contains(word) { score += 2; }
            if title_lower.contains(word) { score += 2; }
        }

        // Exact Match Bonus (+10 points)
        if snippet_lower.contains(&topic.to_lowercase()) { score += 10; }
        if title_lower.contains(&topic.to_lowercase()) { score += 10; }

        // Domain Authority & Restriction
        if let Some(allowed_domains) = domains {
            let matches_domain = allowed_domains.iter().any(|d| url_lower.contains(&d.to_lowercase()));
            if matches_domain {
                score += 50;
            } else {
                score -= 100;
            }
        } else {
            // General heuristics
            if url_lower.contains("github.com") || url_lower.contains("docs.rs") || url_lower.contains("arxiv.org") {
                score += 5;
            }
            if url_lower.contains("quora.com") || url_lower.contains("pinterest.com") || url_lower.contains("zacks.com") {
                score -= 30; // Heavy penalty for common low-signal or marketing-heavy pages
            }
            if url_lower.contains("investor") || url_lower.contains("sec.gov") || url_lower.contains("annualreport") {
                score += 40; // High boost for official investor/regulatory data
            }
        }

        result.score = score;
    }

    // Sort descending by score
    results.sort_by(|a, b| b.score.cmp(&a.score));
}

pub async fn rank_results_with_llm(
    topic: &str,
    results: &mut [DeepResearchSearchResult],
    model: &Arc<dyn language_model::LanguageModel>,
    async_cx: &mut gpui::AsyncApp,
) -> Result<()> {
    if results.is_empty() { return Ok(()); }

    let candidate_count = results.len().min(40);
    let mut candidates_info = Vec::new();
    for (idx, res) in results.iter().take(candidate_count).enumerate() {
        candidates_info.push(format!("[{}] Title: {}\nURL: {}\nSnippet: {}\n", idx, res.title, res.url, res.snippet));
    }

    let mut prompt = format!("You are a world-class research analyst. Your task is to rank the following search results for the research topic: '{}'.\n\n", topic);
    prompt.push_str("Pick the top 12 most authoritative and high-signal sources. Prioritize:\n");
    prompt.push_str("1. Official company documents (Investor Relations, SEC filings).\n");
    prompt.push_str("2. Primary technical documentation or direct source code repositories.\n");
    prompt.push_str("3. Authoritative, data-rich analysis (avoid generic marketing summaries like Zacks/SeekingAlpha).\n\n");
    prompt.push_str("CANDIDATES:\n");
    
    for info in candidates_info {
        prompt.push_str(&info);
        prompt.push_str("\n");
    }
    
    prompt.push_str("\nRespond ONLY with a YAML-style list of indices of the top 12 results in order of relevance, like this:\nindices:\n  - 5\n  - 2\n  - 10\n...");

    let request = language_model::LanguageModelRequest {
        messages: vec![language_model::LanguageModelRequestMessage {
            role: language_model::Role::System,
            content: vec!["You are an expert at identifying high-fidelity information sources. Respond only with the requested YAML structure.".into()],
            cache: false,
            reasoning_details: None,
        }, language_model::LanguageModelRequestMessage {
            role: language_model::Role::User,
            content: vec![prompt.into()],
            cache: false,
            reasoning_details: None,
        }],
        ..Default::default()
    };

    let mut response_text = String::new();
    if let Ok(mut stream) = model.stream_completion_text(request, async_cx).await {
        while let Some(chunk) = stream.stream.next().await {
            if let Ok(text) = chunk {
                response_text.push_str(&text);
            }
        }
    }

    if response_text.is_empty() {
        bail!("LLM returned no response for ranking");
    }
    
    let mut ranked_indices = Vec::new();
    for line in response_text.lines() {
        let trimmed = line.trim();
        // Extract indices from lines like "- 5", "5", "  - 5", "index: 5"
        let parts: Vec<&str> = trimmed.split(|c: char| !c.is_numeric()).filter(|s| !s.is_empty()).collect();
        for part in parts {
            if let Ok(idx) = part.parse::<usize>() {
                if idx < candidate_count && !ranked_indices.contains(&idx) {
                    ranked_indices.push(idx);
                }
            }
        }
    }

    if ranked_indices.is_empty() {
        bail!("LLM returned no valid indices");
    }

    for r in results.iter_mut() {
        r.score = 0;
    }

    for (rank, &idx) in ranked_indices.iter().enumerate() {
        if idx < results.len() {
            results[idx].score = 100 - (rank as i32 * 5);
        }
    }
    
    results.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(())
}

fn parse_results(html: &str) -> Vec<DeepResearchSearchResult> {
    let document = Html::parse_document(html);
    let Ok(result_selector) = Selector::parse(".result") else { return Vec::new(); };
    let Ok(title_selector) = Selector::parse(".result__a") else { return Vec::new(); };
    let Ok(snippet_selector) = Selector::parse(".result__snippet") else { return Vec::new(); };

    let mut results = Vec::new();
    for (idx, element) in document.select(&result_selector).enumerate() {
        let title = element.select(&title_selector).next().map(|el| el.text().collect::<String>()).unwrap_or_default().trim().to_string();
        let mut url = element.select(&title_selector).next().and_then(|el| el.value().attr("href")).unwrap_or_default().to_string();
        let snippet = element.select(&snippet_selector).next().map(|el| el.text().collect::<String>()).unwrap_or_default().trim().to_string();

        if !title.is_empty() && !url.is_empty() {
             // Unmask DuckDuckGo redirects (e.g., /l/?kh=-1&uddg=https%3A%2F%2Fexample.com)
             if url.contains("/l/") && url.contains("uddg=") {
                 if let Some(pos) = url.find("uddg=") {
                     let rest = &url[pos + 5..];
                     let end = rest.find('&').unwrap_or(rest.len());
                     if let Ok(decoded) = url_decode(&rest[..end]) {
                         url = decoded;
                     }
                 }
             }
             
             // Ensure absolute URLs
             if url.starts_with("//") {
                 url = format!("https:{}", url);
             } else if url.starts_with("/") {
                 url = format!("https://duckduckgo.com{}", url);
             }

            results.push(DeepResearchSearchResult { id: idx, title, url, snippet, score: 0 });
        }
    }
    results
}

fn url_encode(input: &str) -> String {
    url::form_urlencoded::byte_serialize(input.as_bytes()).collect()
}

fn url_decode(input: &str) -> Result<String> {
    percent_encoding::percent_decode_str(input)
        .decode_utf8()
        .map(|c: std::borrow::Cow<'_, str>| c.to_string())
        .map_err(|e| anyhow::anyhow!("URL Decode Error: {}", e))
}
async fn browse_parallel(
    http_client: Arc<HttpClientWithUrl>, 
    urls: Vec<String>,
    event_stream: Option<ToolCallEventStream>,
) -> Result<Vec<String>> {

    let mut builder = BrowserConfig::builder();
    builder = builder.no_sandbox();
    
    // Create a unique temporary user data directory to avoid "SingletonLock" errors
    // when running multiple researches or when a previous session didn't clean up.
    let user_data_dir = std::env::temp_dir().join(format!("zed-research-{}", uuid::Uuid::new_v4()));
    builder = builder.user_data_dir(user_data_dir);

    let modern_ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
    builder = builder
        .window_size(1920, 1080)
        .arg(format!("--user-agent={}", modern_ua))
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--disable-extensions")
        .arg("--no-first-run")
        .incognito();
    
    // Attempt to find Chrome on macOS if it's not in the PATH
    #[cfg(target_os = "macos")]
    {
        let common_paths = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        ];
        for path in common_paths {
            if std::path::Path::new(path).exists() {
                builder = builder.chrome_executable(path);
                break;
            }
        }
    }

    let config = builder.build().map_err(|e| anyhow::anyhow!("{e}"))?;
    
    match Browser::launch(config).await {
        Ok((browser, mut handler)) => {
            let _handler_task = tokio::spawn(async move {
                while let Some(_event) = handler.next().await {
                    log::trace!("Browser event: {:?}", _event);
                }
            });

            let browser = Arc::new(browser);
            let mut tab_futures = Vec::new();
            let urls_for_log = urls.clone();
            
            for url in urls.into_iter() {
                let b = browser.clone();
                let event_stream = event_stream.clone();
                let future = async move {
                    if let Some(event_stream) = &event_stream {
                        event_stream.update_fields(acp::ToolCallUpdateFields::new().title(format!("Fetching {}...", url)));
                    }
                    
                    let fetch_result = tokio::time::timeout(std::time::Duration::from_secs(45), async {
                        let page = match b.new_page(&url).await {
                            Ok(p) => p,
                            Err(e) => return Ok::<String, anyhow::Error>(format!("(Failed to open page for {}: {})", url, e)),
                        };

                        // Match BrowserTool's navigation logic
                        let _ = page.wait_for_navigation().await;
                        tokio::time::sleep(std::time::Duration::from_millis(6000)).await;
                        
                        // Scroll down to trigger lazy-loading and move past some overlays
                        let _ = page.evaluate("window.scrollBy(0, 1500)").await;
                        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                        
                        let html_body = match page.content().await {
                            Ok(h) => h,
                            Err(e) => {
                                 let _ = page.close().await;
                                 return Ok::<String, anyhow::Error>(format!("(Failed to get content for {}: {})", url, e));
                            }
                        };
                        
                        let markdown = match html_to_clean_markdown(html_body.as_bytes(), &url) {
                            Ok(m) => m,
                            Err(e) => {
                                 let _ = page.close().await;
                                 return Ok::<String, anyhow::Error>(format!("(Failed to parse markdown for {}: {})", url, e));
                            }
                        };
                        
                        let _ = page.close().await;
                        
                        if markdown.trim().len() < 200 {
                             return Ok::<String, anyhow::Error>(format!("(Page content from {} too thin ({} chars) - possible bot detection)", url, markdown.trim().len()));
                        }
                        
                        Ok::<String, anyhow::Error>(markdown)
                    }).await;

                    match fetch_result {
                        Ok(res) => res,
                        Err(_) => Ok::<String, anyhow::Error>(format!("(Timeout fetching {} after 45s)", url)),
                    }
                };
                tab_futures.push(future);
            }

            let mut outcomes = Vec::new();
            {
                 use futures::StreamExt as _;
                 // Use .buffered() to maintain order, so urls_for_log[idx] matches the outcome.
                 let mut outcomes_stream = futures::stream::iter(tab_futures).buffered(10);
                 while let Some(outcome) = outcomes_stream.next().await {
                     outcomes.push(outcome);
                 }
            }
            
            let mut results = Vec::new();
            for (idx, outcome) in outcomes.into_iter().enumerate() {
                 let url = &urls_for_log[idx];
                 match outcome {
                     Ok(body) => {
                          results.push(body);
                          if let Some(event_stream) = &event_stream {
                              event_stream.update_fields(acp::ToolCallUpdateFields::new().title(format!("✅ Fetched {}", url)));
                          }
                     }
                     Err(e) => {
                          let error_msg = format!("(Error fetching {}: {})", url, e);
                          results.push(error_msg.clone());
                          if let Some(event_stream) = &event_stream {
                              event_stream.update_fields(acp::ToolCallUpdateFields::new().title(format!("❌ Failed: {}", url)));
                          }
                     }
                 }
            }

            if let Ok(mut browser) = Arc::try_unwrap(browser) {
                let _ = browser.close().await;
            }

            Ok(results)
        }
        Err(e) => {
            log::error!("Browser launch failed: {:#}", e);
            // FALLBACK: Use raw HTTP fetch if browser launch fails
            let mut results = Vec::new();
            for url_str in urls {
                if let Some(event_stream) = &event_stream {
                    event_stream.update_fields(acp::ToolCallUpdateFields::new().title(format!("Fetching {} (HTTP Fallback)...", url_str)));
                }

                let response_result = match http_client::Url::parse(&url_str) {
                    Ok(parsed_url) => {
                        let request = http_client::Builder::new()
                            .uri(parsed_url.to_string())
                            .method(http_client::Method::GET)
                            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
                            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
                            .header("Accept-Language", "en-US,en;q=0.9")
                            .follow_redirects(http_client::RedirectPolicy::FollowAll)
                            .body(http_client::AsyncBody::empty());

                        match request {
                            Ok(req) => http_client.send(req).await,
                            Err(e) => Err(anyhow::anyhow!("Request Builder Error for {}: {}", url_str, e)),
                        }
                    }
                    Err(e) => Err(anyhow::anyhow!("Invalid URL {}: {}", url_str, e)),
                };

                match response_result {
                    Ok(mut response) => {
                        let status = response.status();
                        if !status.is_success() {
                             results.push(format!("(Server returned HTTP {} for {})", status, url_str));
                             continue;
                        }
                        
                        let mut body = Vec::new();
                        if response.body_mut().read_to_end(&mut body).await.is_ok() {
                            if let Ok(markdown) = html_to_clean_markdown(&body, &url_str) {
                                if markdown.trim().len() > 100 {
                                    results.push(markdown);
                                    continue;
                                }
                            }
                        }
                        results.push(format!("(Fetched content from {} but it was unreadable or too thin - possible bot protection)", url_str));
                    }
                    Err(err) => {
                        results.push(format!("(HTTP Fetch Error: {})", err));
                    }
                }
            }
            Ok(results)

        }
    }
}

fn html_to_clean_markdown(html_body: &[u8], _url: &str) -> Result<String> {
    let mut handlers: Vec<TagHandler> = vec![
        Rc::new(RefCell::new(markdown::WebpageChromeRemover)),
        Rc::new(RefCell::new(markdown::ParagraphHandler)),
        Rc::new(RefCell::new(markdown::HeadingHandler)),
        Rc::new(RefCell::new(markdown::ListHandler)),
        Rc::new(RefCell::new(markdown::TableHandler::new())),
        Rc::new(RefCell::new(markdown::StyledTextHandler)),
        Rc::new(RefCell::new(markdown::CodeHandler)),
    ];

    convert_html_to_markdown(html_body, &mut handlers)
}
