# 🌐 Headless Chromium Web Search

The integrated web search engine allows the Zed agent to access the live internet, bypassing the static knowledge cutoff of modern Large Language Models.

## The Headless Engine

Unlike simple text-scrapers that fail on modern documentation sites (React, Next.js, Docusaurus), this fork utilizes the **Chromiumoxide** library to instantiate a localized, headless Chromium instance. 

This ensures:
- **Full JavaScript Execution**: Rendered code snippets and dynamic content are fully visible.
- **DOM-to-Markdown Transformation**: After rendering, we traverse the DOM to extract clean, hierarchical Markdown for the agent's context.

## Navigation & Extraction Process

The agent utilizes two tools under the hood:
1.  **`search`**: Queries the DuckDuckGo API to find relevant URLs.
2.  **`browser`**: Navigates to a specific URL, waits for hydrations, and extracts the content.

## Visual Walkthrough

### Stage 1: Discovering the Solution

<ZedChat>
  <template #user>
    How do I fix Rust error E0277 in the latest version of Axum?
  </template>
  <template #assistant>
    I need to search the live web as my training data doesn't cover the most recent Axum release.
  </template>
  <template #output>
    @search({ query: "axum rust error E0277 trait bound not satisfied" })
    >> Result 1: https://docs.rs/axum/latest/axum/error_handling/index.html
    >> Result 2: https://github.com/tokio-rs/axum/issues/1234
  </template>
</ZedChat>

### Stage 2: Deep Extraction

<ZedChat>
  <template #assistant>
    The docs.rs page looks most relevant. I am now navigating to the page to extract the precise fix.
  </template>
  <template #output>
    @browser({ action: "navigate", target: "https://docs.rs/axum/latest/axum/error_handling/index.html" })
    >> Status: Page Rendered (JS Hydrated)
    >> Extracted: [Comprehensive Markdown summary of the fix...]
  </template>
</ZedChat>

## Technical Details

- **Containerization**: Rendering happens entirely within a local chromium process.
- **Privacy**: No tracking tokens are sent; we mimic standard Chrome headers to bypass basic anti-bot triggers.
- **Code Path**: Implemented in `crates/agent/src/tools/web_search_tool.rs`.

## Workflow Impact
- **Infinite Knowledge**: Bypasses the LLM training cutoff by pulling live data from the web.
- **Multi-Source Synthesis**: The agent can read multiple search results to form a consensus on the best approach.
- **Free & Local**: No need for paid Search API keys (Serper, Tavily). Uses your local machine's Chromium instance.
- **Developer-Focused**: Prioritizes documentation sites (docs.rs, GitHub, MDN) over generic SEO content.
