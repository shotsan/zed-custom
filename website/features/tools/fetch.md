# 🌐 Tool: `@fetch`

Pull specific web pages directly into the chat context using a headless Chromium renderer.

## Technical Details
- **Internal File**: `crates/agent/src/tools/fetch_tool.rs`
- **Logic**: Navigates to a URL, executes JavaScript, and converts HTML to Markdown.

## Usage Example

<ZedChat>
  <template #user>
    Read the installation docs at https://bun.sh/docs/installation
  </template>
  <template #assistant>
    Fetching page content via Chromium...
  </template>
  <template #output>
    @fetch({ url: "https://bun.sh/docs/installation" })
    >> Status: Rendered
    >> Content: [Installation Markdown]
  </template>
</ZedChat>

### Advantages
- **JS-Ready**: Captures content from SPA documentation sites (React, Vue, Docusaurus).
- **Direct Access**: Bypasses search results for known URLs.
