# 🔍 Quick Web Search (DDG HTML)

The Quick Search tool provides the Assistant with instant access to the web for fact-checking, API documentation lookups, and real-time data retrieval.

Unlike complex browser-based tools, this system is optimized for **speed** and **privacy** by using a lightweight, static scraping engine.

---

## ⚡ The HTML Engine

The Assistant utilizes a specialized **DuckDuckGo HTML** scraper to pull in facts. This approach offers several technical advantages:

1.  **Zero-JavaScript Overhead**: By targeting the static HTML version of DuckDuckGo, the tool avoids the latency of loading a full browser engine (Chromium/Webkit).
2.  **Instant Context**: Results are parsed in milliseconds and injected directly into the conversation as clean Markdown.
3.  **Privacy Focused**: Searches are proxied through a privacy-centric gateway, ensuring your local IP and system details are never exposed to search engines.
4.  **Stealth Execution**: Because there is no browser footprint, the search is highly resistant to bot-detection and "Unusual Traffic" flags that often plague other AI search tools.

---

## 🛠️ How to Use

Simply type your query or ask a question that requires up-to-date information:

> **User**: *"What is the latest version of the `sqlez` crate and what changed in the last release?"*

**Assistant Action**:
1.  Triggers the `@search` tool with the query.
2.  Connects to `html.duckduckgo.com` and retrieves the top 10 results.
3.  Parses titles, URLs, and snippets using high-speed CSS selectors.
4.  Provides an answer with clickable citations.

---

## 🛡️ Permission Mode

By default, the IDE respects your privacy settings:
-   **Always Allow**: Grant the Assistant permanent permission to search specific domains or all domains.
-   **Ask Always**: The Assistant will request a one-click confirmation before performing any external network request.

---

## Comparison: Quick Search vs. Deep Research

| Feature | Quick Search (`/search`) | Deep Research (`/research`) |
| :--- | :--- | :--- |
| **Engine** | DuckDuckGo Static HTML | Multi-Staged Chromium Search |
| **Speed** | Instant (<1s) | Sequential (~20-40s) |
| **Complexity** | Basic fact-checking | Exhaustive technical deep-dives |
| **JavaScript** | Disabled (Lightweight) | Enabled (Full Browser) |
