# 🌐 Tool: `@fetch`

The `@fetch` tool allows the AI agent to pull specific web pages directly into the context window, bypassing the agent's limited training data bounds and ensuring context perfectly aligns with upstream documentation.

## Technical Implementation

Implemented in `crates/agent/src/tools/fetch_tool.rs`, the tool utilizes a headless Chromium instance (via the **Chromiumoxide** library). 

Unlike standard HTTP libraries, `@fetch` ensures:
- **Client-Side Rendering**: Executes JavaScript (React, Vue, etc.) to capture content on modern single-page applications.
- **Markdown Transformation**: Automatically converts the resulting DOM into clean, hierarchical Markdown.

## Usage Example

<ZedChat>
  <template #user>
    Can you write an initial setup script based on the instructions at https://bun.sh/docs/installation ?
  </template>
  <template #assistant>
    Navigating directly to Bun's installation docs...
  </template>
  <template #output>
    @fetch({ url: "https://bun.sh/docs/installation" })
    >> Status: Rendered (JS Hydrated)
    >> Content: [Installation Markdown Extraction]
  </template>
</ZedChat>

## Key Advantages
- **Deep Context**: Allows the agent to use libraries that were released after its knowledge cutoff.
- **Accurate Syntax**: By reading the official docs, the agent avoids deprecated API patterns.
- **Privacy**: All rendering happens within a local Chromium process.
