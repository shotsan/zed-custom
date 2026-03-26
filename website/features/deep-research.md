# Deep Research

Deep Research is a specialized, autonomous browsing and analysis pipeline designed to deliver high-fidelity data extraction from even the most sophisticated websites. Unlike a standard search, it orchestrates a multi-step investigation that mimics a human researcher's workflow.

## How it Works

The tool follows a rigorous, four-stage investigative process:

1.  **Semantic Expansion**: Your request is automatically expanded into multiple targeted search queries across different domains (e.g., technical docs, financial portals, news).
2.  **Autonomous Ranking**: Discovered URLs are prioritized using zed-custom's internal ranking engine, which scores sources by relevance, domain authority, and keyword overlap.
3.  **Secure Parallel Browsing**: Analyzes the top-ranked sources simultaneously using a fleet of isolated, headless browser tabs. 
4.  **Consolidated Synthesis**: Gathers the extracted data, scrubs out the noise, and presents an "Investigative Status Table" alongside a comprehensive discovery report.

## Built for Resilience

Researching high-security financial portals or technical documentation sites requires a different approach than simple web scraping. 

- **Residential Signature**: Every request uses a native Chromium engine with advanced bot-detection mitigation (via `--disable-blink-features=AutomationControlled`), ensuring you aren't blocked by standard bot-mitigation services.
- **Fail-Safe Orchestration**: If a single website is down or timing out, the task continues with other sources. You are provided with a complete audit trail of what was retrieved and why any individual source was skipped.
- **Session Sandboxing**: Each task launches with a unique temporary user profile, preventing profile lock conflicts and ensuring your research remains private and isolated.

## Using Deep Research

To trigger a deep investigation, simply ask the agent to "Deep research" or "Exhaustively analyze" a topic. You can see the research progress live in the Agent Panel:

- **Fetch Status**: Real-time status for every source being crawled.
- **Audit Table**: A complete list of all analyzed, skipped, and discovered candidate URLs for full transparency.
- **Condensed Synthesis**: A final structured report that you can then used to build upon.

Elevate your AI interactions with data retrieved directly from the live web, delivered with unrivaled accuracy and stealth.
