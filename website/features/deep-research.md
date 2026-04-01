# Deep Research

Deep Research is a specialized, autonomous browsing and analysis pipeline designed to deliver high-fidelity data extraction from even the most sophisticated websites. Unlike a standard search, it orchestrates a multi-step investigation that mimics a human researcher's workflow.

## How it Works

The tool follows a rigorous, four-stage investigative process:

1.  **Semantic Expansion**: Your request is automatically expanded into 6 highly targeted search queries.
2.  **Autonomous Ranking**: Discovered URLs are prioritized using an internal ranking engine, which scores sources by relevance and keyword overlap.
3.  **Secure Parallel Browsing**: Analyzes the top 10-20 sources simultaneously using a fleet of isolated, Chromium tabs. Choose between **Headless (Background)** or **Headed (Visible)** mode for full control.
4.  **Consolidated Synthesis**: Gathers the extracted data, identifies information gaps, and presents a comprehensive discovery report.

## Choosing Your Search Engine

Deep Research can be configured to use different search indices to find initial source candidates. You can toggle between these in your `settings.json`:

### 🦆 DuckDuckGo (Default)
Ideal for **speed and privacy**. It provides a configuration-free, zero-setup experience for most baseline technical research.

### 🌐 Google Search (Advanced Depth)
The premium choice for **high-fidelity discovery**. 
- **Sequential Stealth Mode**: Navigates Google's SERPs with a 1.2s delay ("jitter") between queries to mimic human behavior and avoid bot triggers.
- **Dynamic Challenge Resolution**: Supports **Visible Window** mode to natively resolve "Unusual Traffic" or "Enable JS" challenges.
- **Deep Coverage**: Leverages the world’s most comprehensive index for niche technical documentation and academic whitepapers.
- **No API Keys**: Works entirely via your local Chromium engine.

## Configuration Guide

To enable the specialized Google Search provider, add the following to your `settings.json`:

```json
{
  "agent": {
    "deep_research": {
      "search_provider": "google",
      "use_headed_browser": false,
      "max_concurrent_tabs": 10,
      "max_depth": 3
    }
  }
}
```

### Browser Visibility (Headed vs. Headless)
You can now control whether the investigation happens in the background or in a visible window:
- **Headless (Hidden)**: Runs silently in the background. Best for performance and minimizing distractions.
- **Headed (Visible Window)**: Launches a real Chrome window. Use this if you encounter persistent CAPTCHAs or want to audit the scraper's navigation in real-time.

## Built for Resilience

- **Residential Signature**: Every request uses a native Chromium engine with advanced bot-detection mitigation (via `--disable-blink-features=AutomationControlled`).
- **Fail-Safe Orchestration**: If a single website is down or timing out, the task continues with other sources. You are provided with a complete audit trail.
- **Session Sandboxing**: Each task launches with a unique temporary user profile, preventing profile lock conflicts and ensuring your research remains private and isolated.

## Using Deep Research

To trigger a deep investigation, simply ask the agent to "Deep research" or "Exhaustively analyze" a topic. You can see the research progress live in the Agent Panel:

- **Fetch Status**: Real-time status for every source being crawled.
- **Audit Table**: A complete list of all analyzed, skipped, and discovered candidate URLs for full transparency.
- **Condensed Synthesis**: A final structured report that you can then used to build upon.

Elevate your AI interactions with data retrieved directly from the live web, delivered with unrivaled accuracy and stealth.
