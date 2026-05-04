# Deep Research

Deep Research is a specialized, autonomous browsing and analysis pipeline designed to deliver high-fidelity data extraction from even the most sophisticated websites. Unlike a standard search, it orchestrates a multi-step investigation that mimics a human researcher's workflow.

## How it Works

The tool follows a rigorous, multi-stage investigative process:

1. **Semantic Expansion**: Your request is automatically expanded into 6 highly targeted search queries using the active language model.
2. **Search & Deduplication**: Discovered URLs are collected from your configured search provider and deduplicated.
3. **Autonomous Ranking**: Candidates are prioritized using an LLM-based ranking engine (or heuristic fallback) to score sources by relevance.
4. **Iterative Parallel Browsing**: Analyzes the top sources simultaneously in a fleet of isolated, Chromium tabs (up to `max_concurrent_tabs`). This process repeats for up to `max_depth` iterations.
5. **Gap Analysis**: Between iterations, the agent reads the collected content, identifies 3 critical information gaps, and launches highly specific follow-up queries.
6. **Consolidated Synthesis**: Gathers the extracted data and condenses it into a comprehensive discovery report.

## Search Providers

Deep Research supports several search providers. You can configure them in your `settings.json` under `agent.deep_research`.

### 🦆 DuckDuckGo (Default)
Ideal for **speed and privacy**. It provides a configuration-free experience, using an API with an automatic fallback to browser-based scraping if rate-limited.
*Provider string: `"duckduckgo"`*

### 🌐 Google Search
The premium choice for **high-fidelity discovery**, leveraging the world's most comprehensive index.
- **Sequential Stealth Mode**: Navigates Google's SERPs with a randomized jitter (delay) between queries to mimic human behavior and avoid bot triggers.
- **Dynamic Challenge Resolution**: Works well with `use_headed_browser` to natively resolve "Unusual Traffic" or "Enable JS" challenges.
*Provider string: `"google"`*

### 🔑 API-Based Providers
You can also use dedicated search APIs by providing the respective key:
- **Serper.dev**: `"serper"` (Requires `serper_api_key`)
- **Tavily AI**: `"tavily"` (Requires `tavily_api_key`)
- **Exa AI**: `"exa"` (Requires `exa_api_key`)
- **Brave Search**: `"brave"` (Requires `brave_api_key`)

## Configuration Guide

You can configure Deep Research behaviors, limits, keys, and prompts in your `settings.json`:

```json
{
  "agent": {
    "deep_research": {
      "search_provider": "google",
      "use_headed_browser": false,
      "max_concurrent_tabs": 10,
      "max_depth": 3,
      
      // Optional API Keys for other providers
      "tavily_api_key": null,
      "serper_api_key": null,
      "exa_api_key": null,
      "brave_api_key": null,

      // Optional persistent browser configuration
      "browser_user_data_dir": null,
      "browser_profile": null,

      // Custom System Prompts
      "search_system_prompt": null,
      "gap_analysis_system_prompt": null,
      "condensation_system_prompt": null
    }
  }
}
```

### Browser Visibility (Headed vs. Headless)
You can control whether the investigation happens in the background or in a visible window:
- **Headless (Hidden)**: Runs silently in the background. Best for performance and minimizing distractions.
- **Headed (Visible Window)**: Launches a real Chrome window. Use this if you encounter persistent CAPTCHAs or want to audit the scraper's navigation in real-time.

### Customizing Prompts
You can override the default prompts used at different stages of the research loop:
- `search_system_prompt`: Modifies how the initial topic is expanded into the 6 search queries.
- `gap_analysis_system_prompt`: Modifies the logic for identifying information gaps between scraping iterations.
- `condensation_system_prompt`: Adjusts how the final report is synthesized from the raw content.

## Built for Resilience

- **Residential Signature**: Every request uses a native Chromium engine with stealth mitigations.
- **Fail-Safe Orchestration**: If a single website is down or timing out, the task continues with other sources.
- **Session Sandboxing**: Each task launches with an isolated profile, ensuring your research remains private. You can also supply a custom `browser_user_data_dir` and `browser_profile` for persistent session cookies across research tasks.

## Using Deep Research

To trigger a deep investigation, simply ask the agent to "Deep research" or "Exhaustively analyze" a topic. You can see the research progress live in the Agent Panel:

- **Fetch Status**: Real-time status for every source being crawled.
- **Audit Table**: A complete list of all analyzed, skipped, and discovered candidate URLs for full transparency.
- **Condensed Synthesis**: A final structured report that you can then use to build upon.
