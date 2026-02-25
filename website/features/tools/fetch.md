# 🌐 Tool: `@fetch`

The `@fetch` tool allows the AI agent to pull specific web pages directly into the context window, bypassing the agent's limited training data bounds and ensuring context perfectly aligns with upstream documentation.

## Technical Implementation

Implemented in `crates/agent/src/tools/fetch_tool.rs`, the tool utilizes a headless Chromium instance (via the **Chromiumoxide** library). 

Unlike standard HTTP libraries, `@fetch` ensures:
- **Client-Side Rendering**: Executes JavaScript (React, Vue, etc.) to capture content on modern single-page applications.
- **Markdown Transformation**: Automatically converts the resulting DOM into clean, hierarchical Markdown.

## Usage Examples

### Example 1: Updating out-of-date library syntax
<ZedChat>
  <template #user>
    Update my `Cargo.toml` and main file to use the latest `axum` patterns.
  </template>
  <template #assistant>
    The model has a training cutoff, but I can fetch the live installation docs.
  </template>
  <template #output>
    @fetch({ url: "https://docs.rs/axum/latest/axum/" })
    >> Status: Rendered (JS Hydrated)
    >> Using latest 0.7.x patterns...
  </template>
</ZedChat>

### Example 2: Reading API Specifications
<ZedChat>
  <template #user>
    What are the valid headers for the Anthropic beta prompt caching API?
  </template>
  <template #assistant>
    Fetching Anthropic's developer guide...
  </template>
  <template #output>
    @fetch({ url: "https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching" })
    >> Extracting `anthropic-beta: prompt-caching-2024-07-31` header requirements.
  </template>
</ZedChat>

## Key Advantages & Impact
- **Dynamic Context**: Allows the agent to use libraries that were released after its knowledge cutoff.
- **Accurate Syntax**: By reading the official docs, the agent avoids deprecated API patterns and solves "it works on my machine" issues.
- **Privacy**: All rendering happens within a local Chromium process; no third-party scraping services are used.
- **Bypass Paywalls/Restrictions**: Handles complex login-less modern docs that simple `curl`-based scrapers can't see.
