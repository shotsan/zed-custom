# 🌐 Web Search (Headless Chromium)

Execute real-time web searches directly in the chat panel using a localized, headless Chromium instance.

## Execution Flow

### Step 1: Query Discovery
The agent identifies missing info and searches via the DuckDuckGo API.

<ZedChat>
  <template #user>
    How do I fix Rust error E0277 in Axum?
  </template>
  <template #assistant>
    Searching for recent Axum documentation...
  </template>
  <template #output>
    @search({ query: "axum rust error E0277" })
    >> Found: [docs.rs/axum, github.com/tokio-rs/axum/issues]
  </template>
</ZedChat>

### Step 2: Content Extraction
The agent navigates to the best result, rendering JavaScript to ensure full data capture.

<ZedChat>
  <template #assistant>
    Navigating to docs.rs to extract solution...
  </template>
  <template #output>
    @browser({ action: "navigate", target: "https://docs.rs/axum/latest/axum" })
    >> Status: Rendered (JS Hydrated)
    >> Extracted: [Solution Markdown]
  </template>
</ZedChat>

## Why Headless?

- **JS Execution**: Captures React/Next.js rendered content that simple scrapers miss.
- **Bot Resilience**: Mimics human browser patterns to avoid blocks.
- **Privacy**: No external tracking; all rendering happens locally.

## Technical Superiority over Standard IDEs

Unlike other "AI-powered" editors that use simple text-based scrappers, our headless engine ensures:

*   **JavaScript Execution**: We see exactly what you see in Chrome. If the documentation uses a single-page app (SPA) framework like Next.js or Docusaurus, we capture the content accurately.
*   **Anti-Bot Resilience**: By mimicking human browsing patterns and headers, we can access resources that typically block automated scrapers.
*   **Markdown Conversion Logic**: Once the page is rendered, our custom transformer converts the DOM into a clean, hierarchical Markdown format, preserving code block languages and table structures for the agent's consumption.
