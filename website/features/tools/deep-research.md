# 🧪 Deep Research: Advanced Multi-Stage Investigation

Deep Research is an autonomous, multi-stage recursive engine designed for high-stakes technical investigations, market analysis, and exhaustive fact-finding. It identifies missing data points and performs follow-up searches to close information gaps in every investigation.

---

## 🏗️ The 4-Stage Lifecycle

The tool follows a deterministic state machine to ensure exhaustive coverage of the topic:

### 1. Search Query Expansion
The engine begins by using a high-fidelity model to expand your initial prompt into **~6 highly specific search queries**. 

To ensure broad coverage, the engine purposefully targets five distinct data categories:
1.  **Official Corporate Records**: Investor Relations, Annual Reports, and SEC filings.
2.  **Technical Resources**: Specifications, Documentation, and GitHub repositories.
3.  **Market Analytics**: Financial analysis and industry providers.
4.  **Current Events**: Recent press releases and news.
5.  **Technical Assessment**: Expert critiques and whitepapers.

---

### 2. Search Discovery & Filtering (Phase 1)
The engine detects your configured **Search Provider** and routes to one of two entirely separate logic paths for **URL discovery**. The goal of this phase is to rapidly identify candidate links using specialized scouts.

### 3. Deep Page Analysis & Processing (Phase 2)
Once candidate URLs are discovered, the engine launches a **unified, parallelized Chromium engine** to analyze each page. Choose between **Headless** (Background) or **Headed** (Visible Window) mode to bypass hardware-level fingerprinting or manually resolve traffic challenges. This browser-based scraping allows the tool to process modern, JavaScript-heavy documentation into clean Markdown.

### 4. Recursive Gap Analysis (Discovery Loops)
After gathering initial findings, the engine identifies **critical information gaps** and performs follow-up discovery:
- **Autonomous Query Generation**: The engine automatically writes **3 targeted follow-up queries** (Gaps) based on missing data points.
- **Iteration Depth**: Controlled by the `max_depth` setting. A depth of 3 means the engine will perform up to 3 recursive loops to find missing facts before synthesis.

---

## ⚡ Search Discovery: DuckDuckGo Scout

The DuckDuckGo implementation acts as a **high-speed discovery scout** for initial URL retrieval.

- **URL Discovery Only**: Targets `html.duckduckgo.com` (Zero-JS) to retrieve long lists of candidate links without browser overhead.
- **Direct Scraping**: This phase does **not** use a browser; it utilizes high-performance Rust-based CSS selectors.
- **Massive Parallelism**: Search queries are fired in a single parallel burst to populate the candidate pool instantly.

---

## 🕵️ Search Discovery: Google Search Scout (Stealth Mode)

The Google implementation acts as a **Stealth Scout** designed to navigate defensive security layers and resolve high-fidelity results.

- **Chromium-Driven Search**: Uses a Chromium engine even for the initial result discovery. Supported in both background and visible modes.
- **Sequential Stealth Mode**: Searches are performed **one by one** with a mandatory **1.2-second jitter delay** between queries to avoid traffic flagging.
- **Dynamic Redirect Resolution**: Natively detects and follows Google's "Enable JS" and "Unusual Traffic" redirects by monitoring tab context switching.
- **Broad Bucket Collection**: Acts as a pure data harvester, capturing every link and metadata field before delegating relevance ranking to the semantic engine.

---

## 🚀 The Multi-Tab Browsing Engine (Unified)

Regardless of the search provider, the **Deep Page Analysis** phase is unified and uses a specialized Chromiumoxide engine.

- **Native Challenge Recovery**: Auto-detects secondary tabs spawned by JS-challenges (e.g., `enablejs` redirects), ensuring the scraper always binds to the active result context.
- **Profile Isolation**: Each research session uses a unique, **process-level temporary directory** (`zed-research-profile-<PID>`), preventing SingletonLock conflicts and ensuring private, state-free browsing for every task.
- **Content Conversion**: Every page is analyzed, scrolled to trigger lazy-loading, and converted from raw HTML into **Technical Markdown**.

---

## 🤖 Customizing the System Prompts

Advanced users can refine the internal logic used during each stage by overriding these prompts in their settings:

1.  **`search_system_prompt`**: Controls the initial search query expansion strategy.
2.  **`gap_analysis_system_prompt`**: Controls how missing information is identified and follow-up queries are written.
3.  **`condensation_system_prompt`**: Controls the final report's tone, structure, and technical density.
4.  **`use_headed_browser`**: Toggles between a hidden background browser and a visible window for manual audit and CAPTCHA resolution.
