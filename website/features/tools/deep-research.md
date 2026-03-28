# 🕵️ Deep Research: The Discovery Engine

The **Deep Research Tool** is not just a search tool—it is an **autonomous investigative agent** built directly into Zed. While standard search tools (like `/search` or `/fetch`) provide immediate answers to simple questions, Deep Research is designed for the high-uncertainty, multi-source investigation required by senior engineers and researchers.

It transforms the agent from a static knowledge base into a **recursive discovery engine** that hunts for information across the live web, identifies its own blind spots, and synthesizes a comprehensive report.

---

## 🔁 The Recursive Loop: How it "Thinks"

The "Deep" in Deep Research comes from its **recursive gap analysis**. Unlike a single-pass search, the agent actively critiques its own findings to find what is missing.

### Stage 1: Topic Expansion (The Horizontal Sweep)
The agent takes your objective (e.g., *"How does the GPU pipeline work in GPUI?"*) and expands it into 6-10 distinct search queries. It doesn't just search for the query; it searches for:
- Official documentation and whitepapers.
- GitHub implementation details.
- Developer community discussions and edge cases.
- Recent changes or regressions in changelogs.

### Stage 2: Parallel Headless Crawling
Using a localized, headless Chromium instance, the agent launches up to **10 tabs in parallel**. This isn't just a text scrape; the engine:
- Executes JavaScript to render dynamic content.
- Mimics modern browser headers to bypass basic anti-bot triggers.
- Automatically handles redirects and hydrates content before extracting a clean Markdown representation.

### Stage 3: Gap Analysis (The Deep Dive)
**This is the core differentiator.** After reading the first 10 pages, the agent asks itself: *"Given everything I just read, what is still missing to truly answer the user?"*
- If it finds a library's API but not its performance trade-offs, it spawns new queries specifically targeting benchmarks.
- If it finds a founder's name but not their previous technical exits, it pivots to corporate filings.
- It then crawls these **follow-up targets** in a second iteration.

### Stage 4: Synthesis & The Discovery Report
Finally, all collected context (often 20,000+ words of raw data) is distilled into a structured Markdown report, complete with source citations and a "Status Table" for full auditability of the research path.

---

## 📖 Real-World Example: Investigating an Entity

When you research a person or a company, the agent doesn't just give you a bio—it maps their digital footprint.

<ZedChat>
  <template #user>
    /deep_research "Santosh Ganji" professional background and technical contributions
  </template>
  <template #assistant>
    🚀 **Iteration 1: Mapping the Surface**
    🔍 Expanded into: "Santosh Ganji LinkedIn", "Santosh Ganji IEEE papers", "Santosh Ganji startup exits", "Santosh Ganji developer profiles"
    🌐 Fetching: IEEE Author Profile... ✅ Analyzed
    🌐 Fetching: Scholar.google citations... ✅ Analyzed
    🌐 Fetching: LinkedIn professional history... ✅ Analyzed
    
    🧠 **Gap Analysis**: "I found his academic citations and current LinkedIn title, but his specific role in the 'Godspeed' project and his earlier corporate directorships in India are unclear."
    
    🚀 **Iteration 2: The Deep Dive**
    🔍 Follow-up: "Santosh Kumar Ganji director profile MyCorporateInfo", "Santosh Ganji Godspeed project technical architecture"
    🌐 Fetching: thecompanycheck.com profile... ✅ Analyzed
    ...
  </template>
  <template #output>
    # Discovery Report: Santosh Ganji
    
    ## 🎓 Academic Profile
    - **Citations**: Extensive work in timbre characteristics and speech features.
    - **Key Paper**: "Speakers' Ethnical Features as a Factor Influencing Timbre..." (IEEE/Gramota).
    
    ## 💼 Professional Tenure
    - **Current**: Leading technical initiatives at [Redacted].
    - **History**: Previously held directorships at [Company A] and [Company B], focused on specialized infrastructure.
    
    ## 🛠 Technical Contributions
    - Deep involvement in the **Godspeed** activity (Activity ID: 715345...), focused on high-scale distributed systems.
    
    | Status | Source Title | URL |
    | :--- | :--- | :--- |
    | ✅ Analyzed | IEEEXplore Author Profile | https://ieeexplore.ieee.org/... |
    | ✅ Analyzed | Scholar Citations | https://scholar.google.com/... |
    | ❌ Failed | Paywalled Bio (Thin Content) | https://zoominfo.com/... |
  </template>
</ZedChat>

---

## 🛠 Advanced Features

### 🛡 Stealth & Reliability
- **Isolated User Directories**: Every research task uses a fresh, temporary Chromium profile. This prevents `SingletonLock` errors and ensures that cookies/cache from one session don't leak into another.
- **Automation Masking**: We remove the `navigator.webdriver` flag and use customized viewport/UA settings to ensure the agent isn't blocked by standard bot-detection.

### ⚡ Live Status Streams
Deep Research in Zed-Custom uses a custom `ToolCallEventStream`. You don't wait for a "Thinking..." icon for 2 minutes—you see every URL the agent is attempting to fetch, every title it resolves, and every gap it identifies **in real-time**.

### 🔄 Thread Persistence
Because research can take time (1-3 minutes for 3-deep iterations), we have stabilized the session management. You can **switch to another thread**, work on code, and come back—the research continues in the background and your logs will be waiting for you.

---

## ⚙️ Configuration

Tune the engine in your `settings.json`:

```json
{
  "agent": {
    "deep_research": {
      "max_concurrent_tabs": 10,
      "max_depth": 3,
      "search_provider": "duckduckgo"
    }
  }
}
```

- **`max_concurrent_tabs`**: How many sources the agent attempts to "win" (10 is the sweet spot for breadth).
- **`max_depth`**: How many recursive iterations (1 = surface, 2 = standard, 3 = deep).
- **`search_provider`**: Choose between `duckduckgo`, `google` (requires API key), or `tavily` (requires API key).
