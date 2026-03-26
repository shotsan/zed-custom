# Deep Research

The `/deep-research` command turns the agent into a multi-stage recursive web research engine. It searches the web, fetches and parses the actual page content, identifies missing information, and launches follow-up searches to fill the gaps—all automatically, before synthesizing everything into a structured Markdown report.

## Usage

Type `/deep-research <topic>` in the agent message editor, where `<topic>` is the subject you want to investigate. You can also use the `deep_research` tool directly from the LLM agent.

**Examples:**

```
/deep-research chromiumoxide async browser automation in Rust
/deep-research React Server Components vs Client Components tradeoffs
/deep-research how tokio handles task scheduling under the hood
```

You can optionally restrict research to specific domains:

```json
{
  "topic": "GPUI rendering pipeline",
  "domains": ["zed.dev", "github.com"]
}
```

## How It Works

The pipeline runs in five sequential stages:

### 1. Query Expansion

The active LLM takes your topic and expands it into **6 distinct search queries**, each targeting a different angle—official docs, GitHub repos, changelogs, community discussions, etc. This ensures the search covers more surface area than a single query would.

If no LLM is active, only the original topic is used as the query.

### 2. DuckDuckGo Search

All 6 queries are executed in **parallel** against DuckDuckGo's HTML search endpoint (`html.duckduckgo.com`). DuckDuckGo redirect URLs are unmasked via percent-decoding so the real destination URLs are used for deduplication and fetching.

After collecting all results, duplicates are removed by normalizing URLs (stripped of query parameters and trailing slashes, lowercased).

### 3. LLM Source Ranking

The active LLM is given up to 40 candidate results (title, URL, snippet) and asked to rank the top 12 most authoritative sources for the topic. The ranking response is parsed to reorder the result pool.

If LLM ranking fails (no model, bad response format), a **heuristic fallback** scores results based on:
- Keyword overlap with the topic in the title and snippet
- Small boosts for `github.com`, `docs.rs`, `arxiv.org`
- Penalties for known low-signal domains (`quora.com`, `pinterest.com`)
- Domain restriction bonuses when specific domains are configured

### 4. Parallel Browser Fetching (Iteration 1)

The top-ranked candidates are fetched using a **local headless Chromium instance**. The browser is launched with:
- `--disable-blink-features=AutomationControlled` to avoid bot-detection flags
- A modern Chrome user-agent string
- Incognito mode and a unique temporary user data directory per research session (to prevent `SingletonLock` conflicts)
- A 1920×1080 viewport

For each URL:
1. The page is opened and navigation is awaited.
2. A 6-second delay allows JS-heavy pages to render.
3. The page is scrolled down 1500px to trigger lazy-loaded content.
4. The HTML is extracted and converted to clean Markdown.
5. Pages with fewer than 200 characters of content are marked as **bot-blocked or thin**.

All fetches run **concurrently using `.buffered(10)`** which maintains the result order, ensuring each fetched result maps correctly to its source URL.

**Fallback:** If the browser fails to launch (Chrome not installed), the tool falls back to direct HTTP fetching using the built-in HTTP client.

A **45-second timeout** applies to each URL. Sites that don't respond are marked as timed out.

### 5. Gap Analysis & Follow-up

After each fetch pass (except the last one), the pipeline asks the LLM: **"Given everything you just read, what's still missing?"** This is gap analysis.

The LLM receives up to 10,000 characters of all content collected so far and must return **3 targeted search queries** to find what wasn't covered. Those queries are searched on DuckDuckGo, the new results are deduplicated, re-ranked, and added to the pool for the next iteration.

This is what makes deep research recursive — each iteration doesn't just fetch more of the same results, it actively hunts for specific missing information identified from what was already found.

#### Concrete example

**Topic:** `/deep-research tokio task scheduling internals`

**Iteration 1 fetches:**
- `docs.rs/tokio` — official API reference
- `tokio.rs` — landing/blog page
- A blog post: "Understanding async Rust"
- A GitHub issue about task starvation
- A StackOverflow thread about `spawn_blocking`

**Gap analysis reads all of the above and returns:**
```
tokio work-stealing thread pool implementation source code
tokio task queue backpressure and blocking thread limits
tokio runtime scheduler benchmarks vs rayon thread pool
```

These are things the initial search results *mentioned* but didn't go deep on. The first pass covered the surface — what tokio is and how to use it. The gap analysis identified that the *internals* (how the work-stealing queue actually works, what happens under backpressure, how it compares in benchmarks) are still missing.

**Iteration 2 then fetches:**
- The actual tokio scheduler source on GitHub
- A benchmarking post comparing runtimes
- A doc page specifically about `spawn_blocking` thread limits

The final synthesis now has both the surface-level API knowledge and the deep implementation details.

Gap analysis runs after each iteration except the last. At `max_depth: 3`, the second round of gap analysis has even more content to reason about, so it can identify progressively narrower and more specific gaps.


### 6. Report Synthesis

The final stage passes all successfully fetched Markdown content (up to 12,000 characters per source) to the LLM with a synthesis prompt. The LLM produces a coherent, multi-source Markdown report.

If the LLM is unavailable, the raw concatenated source data is returned directly.

The output always includes an **Investigative Status Table** showing the fate of every URL that was attempted:

| Status | # | Title | URL |
| :--- | :--- | :--- | :--- |
| ✅ Analyzed | 1 | ... | ... |
| ❌ Failed: (Timeout after 45s) | 2 | ... | ... |
| ⏸️ Unreached | 3 | ... | ... |

## Terminal Telemetry

Every step emits `log::info!` and `log::warn!` entries visible in your terminal:

```
🔍 Deep Research: Executing 7 search queries...
✅ Deep Research: Found 21 raw candidate sources.
✅ Deep Research: Deduplicated to 18 unique sources.
🚀 Deep Research: Iteration 1 starting fetch...
🌐 Deep Research: Attempting to fetch: https://docs.rs/tokio/latest/tokio/
✅ Deep Research: Successfully analyzed: https://docs.rs/tokio/latest/tokio/
❌ Deep Research: Blocked or Failed: https://example.com/paywalled ((Page content too thin (43 chars) - possible bot detection))
🚀 Deep Research: Iteration 2 starting fetch...
```

## Configuration

All settings live under `agent.deep_research` in your `settings.json`:

```json
{
  "agent": {
    "deep_research": {
      "max_concurrent_tabs": 10,
      "max_depth": 3,
      "search_provider": "duckduckgo",
      "search_system_prompt": null,
      "gap_analysis_system_prompt": null,
      "condensation_system_prompt": null
    }
  }
}
```

### Parameters

| Parameter | Default | Description |
| :--- | :--- | :--- |
| `max_concurrent_tabs` | `10` | Target number of successfully analyzed sources per research task. The pipeline keeps fetching until this count is reached or the source pool is exhausted. |
| `max_depth` | `3` | Reserved for future multi-hop crawling. Currently controls iteration count. |
| `search_provider` | `"duckduckgo"` | Search backend. Currently `"duckduckgo"` is the active provider. |
| `search_system_prompt` | `null` | Override the prompt used to expand the topic into search queries. When `null`, the built-in prompt is used. |
| `gap_analysis_system_prompt` | `null` | Override the prompt used to identify information gaps for follow-up searches. When `null`, the built-in prompt is used. |
| `condensation_system_prompt` | `null` | Override the prompt used to synthesize raw scraped content into the final report. When `null`, the built-in prompt is used. |

## Customizing the Prompts

Each of the three LLM stages uses a built-in default prompt that you can override. Set the value to a string in `settings.json` to replace it.

### Expansion Prompt (`search_system_prompt`)

Controls how the topic is turned into search queries. The LLM receives this as a system message alongside the user message: `"Expand this topic into 6 diverse search queries: '<topic>'"`.

**Built-in default:**
```
You are a world-class investigative research analyst. Your task is to expand the provided topic into 6 highly specific and diverse search queries.
Ensure you target a broad spectrum of data sources:
1. Official Company Statements (Investor Relations, Annual Reports, SEC filings).
2. Technical Specifications or GitHub repositories.
3. Financial and Market Analysis (diverse providers).
4. Recent Press Releases and News.
5. Expert critiques or technical white papers.

Avoid duplicate queries. Each query must target a distinct 'angle' of the topic.
Provide ONLY the queries, one per line, with no extra text or formatting.
```

**Example override for dev tooling topics:**
```json
{
  "agent": {
    "deep_research": {
      "search_system_prompt": "Expand the given topic into 6 focused technical search queries covering: official documentation, GitHub repositories and issues, crate/package registries, changelogs, engineering blog posts, and community discussions (StackOverflow, Reddit). Return only the queries, one per line."
    }
  }
}
```

### Gap Analysis Prompt (`gap_analysis_system_prompt`)

The LLM receives this prompt after Iteration 1, along with up to 10,000 characters of already-collected content. It must return exactly 3 follow-up search queries, one per line.

**Built-in default:**
```
You are a world-class investigative researcher. You have been researching '<topic>'.
Here is the content you have collected so far:

[START COLLECTED CONTENT]
...
[END COLLECTED CONTENT]

Identify 3 critical information gaps or missing specific data points (e.g., hard numbers, specific projections, competitive nuances) that were NOT found in the content above.
Generate 3 highly specific search queries to find this missing information.
Return ONLY the 3 queries, one per line.
```

### Synthesis Prompt (`condensation_system_prompt`)

The LLM receives this as a system message. The user message contains all raw scraped Markdown from successful fetches.

**Built-in default:**
```
You are an expert technical researcher synthesizing deep research data.
Analyze the raw research material from MULTIPLE sources and provide a highly detailed, coherent, and comprehensive Markdown report.
Citing multiple diverse sources is CRITICAL. Do not rely on just one primary source if others are available.
Synthesize cross-source data points to provide the most authoritative view.
```

## Requirements

- **Chrome or Chromium** must be installed on your system. The tool checks the following paths on macOS:
  - `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
  - `/Applications/Chromium.app/Contents/MacOS/Chromium`
  - `/Applications/Brave Browser.app/Contents/MacOS/Brave Browser`
  - `/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge`

- If no browser is found, the tool falls back to plain HTTP fetching. JavaScript-heavy pages will likely return thin content in this mode.

## Known Limitations

- **Bot-protected pages** (paywalls, Cloudflare challenges, heavy JS fingerprinting) will return thin content and be marked `❌ Failed`. These are logged with the exact failure reason.
- **Per-source content cap**: Only the first 12,000 characters of each fetched page are passed to the synthesis LLM.
- **Search provider**: Only DuckDuckGo is currently supported.

## Changing Iterations and Prompts

### Iterations (`max_depth`)

`max_depth` controls how many **fetch-and-analyze passes** the pipeline runs. Each pass takes a batch of candidates from the ranked pool, fetches them in parallel via the browser, and adds the successful ones to the results.

**The loop has three early-exit conditions** — it stops as soon as any of these are true:
1. The iteration count reaches `max_depth`
2. The number of successfully analyzed sources reaches `max_concurrent_tabs`
3. The candidate pool is empty

```json
{
  "agent": {
    "deep_research": {
      "max_depth": 2
    }
  }
}
```

#### What happens at each depth level

**`max_depth: 1` — Single pass, no gap analysis**

```
Expand topic → Search DDG → Rank → Fetch batch → Synthesize
```

The pipeline fetches the top-ranked sources once and synthesizes. No follow-up searches happen.

---

**`max_depth: 2` — Default. One gap analysis round**

```
Expand topic → Search DDG → Rank → Fetch batch (iter 1)
  → LLM identifies gaps → 3 follow-up searches → new candidates added and re-ranked
  → Fetch second batch (iter 2) → Synthesize
```

After iteration 1, the LLM reads what was collected, identifies missing information, and generates 3 targeted follow-up search queries. Those results are deduplicated, re-ranked, and added to the pool. Iteration 2 fetches from this enriched pool.

---

**`max_depth: 3` — Two gap analysis rounds**

```
Fetch batch (iter 1) → gap analysis → new candidates
Fetch batch (iter 2) → gap analysis → more new candidates
Fetch batch (iter 3) → Synthesize
```

Gap analysis runs after every iteration **except the last**. With depth 3, you get two rounds of gap identification—the second one has more source material to work with, so it can identify gaps the first round couldn't. Each round adds new DDG searches and candidates to the pool.

---

**`max_depth: N` — N-1 gap analysis rounds**

The general pattern: gap analysis fires after iterations 1 through N-1. Each round reads all content collected so far and generates 3 new targeted queries. Higher depth gives the LLM more context to identify progressively more specific gaps.

**Tradeoffs at high depth:**
- Each extra iteration adds browser fetch latency (6s wait + page load, up to 45s per URL)
- Each gap analysis round costs an LLM call
- More source content is passed to the synthesis LLM → higher token usage
- Returns diminish at high depth: the pool is re-ranked each round, so the best sources are already fetched early

The number of **follow-up queries per gap analysis round** (currently 3) is embedded in the gap analysis prompt. To change it, override `gap_analysis_system_prompt` in settings and adjust the number in the prompt text.

---

### Prompts (via `settings.json`)

All three prompts can be overridden without touching the source. Edit your `settings.json`:

**macOS:** `~/.config/zed/settings.json`

```json
{
  "agent": {
    "deep_research": {
      "search_system_prompt": "Your custom expansion prompt here",
      "gap_analysis_system_prompt": "Your custom gap analysis prompt here",
      "condensation_system_prompt": "Your custom synthesis prompt here"
    }
  }
}
```

Set any of these to `null` (or omit them entirely) to revert to the built-in default for that stage.

---

### Prompts (in source code)

If you want to change the built-in defaults permanently (i.e., the fallback when no setting is configured), edit the hardcoded strings directly:

| Prompt | Function | File |
| :--- | :--- | :--- |
| Expansion | `expand_topic()` | `crates/agent/src/tools/deep_research_tool.rs` |
| Gap Analysis | `identify_gaps_and_relaunch()` | `crates/agent/src/tools/deep_research_tool.rs` |
| Synthesis | `condense_report()` | `crates/agent/src/tools/deep_research_tool.rs` |

All three functions accept a `custom_prompt: Option<&str>` argument. When `Some`, the setting value is used. When `None`, the hardcoded `unwrap_or(...)` default runs. To change a default, edit that `unwrap_or(...)` string literal in the respective function.
