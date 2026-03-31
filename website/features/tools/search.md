# 🌐 Headless Chromium Web Search

The integrated web search engine allows the Zed agent to access the live internet, bypassing the static knowledge cutoff of modern Large Language Models.

## The Headless Engine

Unlike simple text-scrapers that fail on modern documentation sites (React, Next.js, Docusaurus), this fork utilizes the **Chromiumoxide** library to instantiate a localized, headless Chromium instance. 

This ensures:
- **Full JavaScript Execution**: Rendered code snippets and dynamic content are fully visible.
- **DOM-to-Markdown Transformation**: After rendering, we traverse the DOM to extract clean, hierarchical Markdown for the agent's context.

## Navigation & Extraction Process

The agent utilizes two primary tools under the hood:
1.  **`search`**: Queries search providers (DuckDuckGo or Google) to find relevant URLs.
2.  **`browser`**: Navigates to a specific URL, waits for hydrations, and extracts the content.

### Google Search Capability
In addition to DuckDuckGo, the agent supports a robust browser-based Google search implementation. This uses customized user-agents and headless Chromium profiles to browse Google’s search result pages directly, ensuring high availability and bypassing aggressive bot detection for a rich discovery experience.

## Technical Details

- **Containerization**: Rendering happens entirely within a local chromium process.
- **Privacy**: We mimic standard Chrome headers to bypass basic anti-bot triggers.
- **Code Path**: Implemented in `crates/agent/src/tools/web_search_tool.rs` and `crates/agent/src/tools/deep_research_tool.rs`.

## Workflow Impact
- **Infinite Knowledge**: Bypasses the LLM training cutoff by pulling live data from the web.
- **Multi-Source Synthesis**: The agent can read multiple search results to form a consensus on the best approach.
- **Free & Local**: No need for paid Search API keys (Serper, Tavily). Uses your local machine's Chromium instance.
- **Developer-Focused**: Prioritizes documentation sites (docs.rs, GitHub, MDN) over generic SEO content.
