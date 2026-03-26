# Deep Research Tool

The Deep Research Tool is a sophisticated, multi-stage data collection and analysis pipeline built directly into zed-custom. It enables the agent to perform autonomous, high-fidelity research on complex topics without user intervention.

## Architectural Overview

The Deep Research tool operates in an intelligent, iterative cycle to ensure maximum coverage and accuracy:

1.  **Expansion Phase**: The agent uses its current LLM to expand your initial topic into a set of optimized, diverse search queries (technical, financial, and news-based).
2.  **Search Phase**: These queries are executed in parallel using a specialized DuckDuckGo search orchestration engine, aggregating hundreds of potential candidates.
3.  **LLM-Powered Ranking**: Unlike basic heuristic search, the tool uses the active language model to analyze candidate titles, URLs, and snippets. It prioritizes authoritative sources (SEC filings, investor relations, technical docs) while deprioritizing low-signal marketing content.
4.  **Parallel Ordered Browsing**: The tool launches a dedicated, headless Chromium instance. Sources are fetched in an **Ordered Concurrent Buffer** (size 10) to ensure high speed without sacrificing data integrity or triggering IP-based bot blocks.
5.  **Iterative Gap Analysis**: After the first pass, the agent analyzes the collected data to identify "Information Gaps." It then generates targeted follow-up queries to hunt for specific missing data points (e.g., hard numbers or competitive nuances).
6.  **Synthesis**: The extracted data is cleaned, converted to Markdown, and synthesized into a structured "Discovery Report."

## Key Features

### Intelligent Failure Recovery
Web research is messy. The tool is designed to be "self-healing":
- **Automatic Quota Filling**: If a priority source fails (due to bot-blocking, 404, or thin content), the tool automatically pivots to the "next best" candidate in the ranked pool until your target source count is reached.
- **Ordered Data Integrity**: Uses ordered concurrency to ensure that search results correctly map to their source metadata, preventing "result drift" in high-speed parallel fetches.

### Advanced Stealth Engine
To access high-security financial and technical websites, the Deep Research tool uses a **Native Browser Signature**:
- **Blink-Automation Removal**: Strips the `navigator.webdriver` flag at the engine level.
- **Natural Interaction**: Simulates human behavior with lazy-load scrolling and stabilized wait times (6s) to ensure JS-heavy dashboards render fully.
- **Unique Session Sandboxing**: Every research task creates a one-time-use temporary User Data Directory (UDD), ensuring session isolation and preventing browser lock conflicts.

## Customizable Research Prompts

Deep Research is driven by three primary LLM interaction stages. You can override any of these prompts in your `settings.json` to tailor the agent's behavior to specific industrial or technical domains.

### 1. Expansion Prompt (`search_system_prompt`)
This prompt is used in the first phase to turn your topic into a search strategy.
**Default:**
```markdown
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

### 2. Gap Analysis Prompt (`gap_analysis_system_prompt`)
Used in the iterative discovery phase to identify missing data after the first fetch batch.
**Default:**
```markdown
You are a world-class investigative researcher. You have been researching '{{topic}}'.
Identify 3 critical information gaps or missing specific data points (e.g., hard numbers, specific projections, competitive nuances) that were NOT found in the content so far.
Generate 3 highly specific search queries to find this missing information.
Return ONLY the 3 queries, one per line.
```

### 3. Synthesis Prompt (`condensation_system_prompt`)
Used to combine the raw scraped material into the final Markdown report.
**Default:**
```markdown
You are an expert technical researcher synthesizing deep research data.
Analyze the raw research material from MULTIPLE sources and provide a highly detailed, coherent, and comprehensive Markdown report.
Citing multiple diverse sources is CRITICAL. Do not rely on just one primary source if others are available.
Synthesize cross-source data points to provide the most authoritative view.
```

## Configuration & Settings

Manage these settings in your `settings.json`:

```json [settings]
{
  "agent": {
    "deep_research": {
      "max_concurrent_tabs": 10,
      "search_system_prompt": "You are a world-class investigative research analyst...",
      "gap_analysis_system_prompt": "You are a world-class investigative researcher. You have been researching '{{topic}}'...",
      "condensation_system_prompt": "You are an expert technical researcher synthesizing deep research data..."
    }
  }
}
```

### Table of Parameters
| Parameter | Default | Description |
| :--- | :--- | :--- |
| `max_concurrent_tabs` | `10` | The target number of successful sources to fetch and analyze. |
| `search_system_prompt` | `None` | Instructions for how the LLM should generate search queries. |
| `condensation_system_prompt`| `None` | Instructions for how the LLM should condense the raw web data into a final report. |

## Observability & Forensic Proof

Every research task generates an **Investigative Status Table** and detailed terminal logs providing full transparency:

- **Analytic Audit Trail**: Explicitly marks each source as:
    - **Fully Analyzed**: The priority sources successfully fetched and parsed.
    - **Retrieval Issue**: Sources that were attempted but failed (with specific error context like 'Thin Content' or 'Timeout').
    - **Targeted Follow-up**: Sources found during the Iterative Gap Analysis phase.
- **Terminal Telemetry**: Real-time logs showing every search attempt, deduplication count, and fetch outcome for forensic verification.

## Best Practices
- **Topic Specificity**: Providing specific tickers or technical terms help the LLM ranker identify the most authoritative technical documentation.
- **Concurrent Research**: Because every task uses a unique temporary profile, you can run multiple deep research threads across different panels without conflict.
