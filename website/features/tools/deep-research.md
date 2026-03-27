# 🔍 Deep Research Tool

The Integrated Deep Research engine transforms the Zed agent into an autonomous, multi-stage investigative researcher. Unlike standard web search, it identifies information gaps, performs recursive follow-up searches, and synthesizes multiple high-fidelity sources into a structured discovery report.

## The Strategy

Deep research follows a recursive strategy to ensure it doesn't just scratch the surface:

1.  **Query Expansion**: Your topic is expanded into 6 diverse search queries targeting different angles (official docs, GitHub, community, news).
2.  **Autonomous Ranking**: Each find is ranked for authority by the LLM (with a heuristic fallback).
3.  **Recursive Discovery**: After the first batch of fetches, the agent performs a "Gap Analysis" to identify what's still missing and launches 3 targeted follow-up queries.
4.  **Local Headless Crawling**: All fetches happen in parallel via a local, headless Chromium instance to bypass basic bot-detection and JS-heavy rendering issues.

## Usage

Type `/deep_research <topic>` in the agent's turn to initiate a research session.

<ZedChat>
  <template #user>
    /deep_research how does the GPU pipeline work in Zed's GPUI framework?
  </template>
  <template #assistant>
    🚀 Expanding topic into search queries...
    🌐 Fetching: GPUI Rendering Engine (zed.dev)
    🌐 Fetching: gpui/src/platform/metal/device.rs (GitHub)
    ...
  </template>
  <template #output>
    # GPUI Rendering Analysis
    
    Found detailed implementation in the Metal and Vulkan backends...
    [Comprehensive Markdown Report Synthesized from 12+ sources]
  </template>
</ZedChat>

## Configuration

You can tune the research depth and concurrency in your `settings.json`:

```json
{
  "agent": {
    "deep_research": {
      "max_concurrent_tabs": 10,
      "max_depth": 3
    }
  }
}
```

## Persistence & Stability

This fork includes custom reliability fixes for deep research:
- **Session Persistence**: Research continues in the background if you switch tabs.
- **Title Tracking**: Progress logs show actual page titles instead of raw URLs.
- **Singleton Isolation**: Uses isolated Chromium profiles per session to prevent lock conflicts.

## Technical Implementation
- **Browser**: Managed via `chromiumoxide` in `crates/agent/src/tools/browser_tool.rs`.
- **Logic**: Orchestrated in `crates/agent/src/tools/deep_research_tool.rs`.
- **UI**: Live streaming events handled by the `AcpThread` entity in `crates/agent/src/thread.rs`.
