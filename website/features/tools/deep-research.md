# 🕵️ Deep Research: The Discovery Engine

The **Deep Research Tool** is an autonomous investigative agent built directly into Zed. It is designed for high-uncertainty, multi-source investigation required by senior engineers and researchers.

---

## 🔁 The Recursive Loop: How it Works

Deep Research follows a recursive "Fetch → Analyze → Gap Analysis → Fetch Again" cycle:

### 1. Semantic Expansion
The agent takes your objective and expands it into **6 highly specific search queries**. It targets documentation, GitHub repositories, and technical whitepapers to ensure broad coverage.

### 2. Parallel Headless Crawling
Using a localized, headless Chromium instance, the agent launches up to **10 tabs in parallel**. It executes JavaScript, handles hydrates, and extracts a clean Markdown representation of the page content.

### 3. Gap Analysis (Recursive Discovery)
After the first sweep, the agent critiques its own findings: *"What is still missing to truly answer the user?"* It then generates targeted follow-up queries and launches a second iteration of searches to fill those gaps.

### 4. Synthesis & The Discovery Report
Finally, the collected context (often 20,000+ words) is distilled into a structured Markdown report with source citations and an audit status table.

---

## 🎯 Mastering Google-Powered Research

While DuckDuckGo is excellent for speed, the **Google Search Engine** is the recommended choice for high-fidelity technical discovery. 

### Why Use the Google Engine?
- **Sequential Stealth Mode**: Unlike parallel search engines, the agent navigates Google results sequentially with built-in "jitter" (1.2s delay between queries) to mimic human browsing and bypass aggressive bot detection.
- **Structural Analysis**: The agent uses tag-based structural analysis to unmask Google's redirect URLs, ensuring it extracts the highest-quality destination sources.
- **Niche Discovery**: Accesses the world's most comprehensive index for deep technical whitepapers and academic filings that generic engines often miss.

### How to Configure Google Search
To enable the Google search provider, update your `settings.json`:

```json
{
  "agent": {
    "deep_research": {
      "search_provider": "google",
      "max_concurrent_tabs": 10,
      "max_depth": 3
    }
  }
}
```

- **`search_provider`**: Set to `"google"` (default is `"duckduckgo"`).
- **`max_concurrent_tabs`**: Number of sources the agent analyzes per iteration (default: 10).
- **`max_depth`**: Number of recursive "gap-filling" iterations (1-3).

---

## 🛡 Stealth & Privacy

- **Isolated Profiles**: Every research task uses a fresh, temporary Chromium user profile, preventing session conflicts and protecting your privacy.
- **Zero API Keys**: Google search works natively through your local Chromium engine. No expensive Serper or Tavily API keys are required.
- **Automation Masking**: Typical automation flags are removed from the browser to ensure high reliability across documentation sites.

---

## ⚡ Thread Persistence

Deep Research is designed for the long haul. Because deep investigations can take 1-3 minutes, you can safely **switch to other files or threads**—the research runs in the background and your final report will be waiting for you.
