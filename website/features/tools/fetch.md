# 🌐 Tool: `@fetch`

The `fetch` tool allows the agent to pull specific web pages directly into the context window, bypassing the AI's limited training data bounds and ensuring context perfectly aligns with upstream documentation.

## Technical Details

**Internal File:** `crates/agent/src/tools/fetch_tool.rs`

When you mention a URL starting with `https://...` or explicitly command the agent to read a webpage, it spawns a headless Chromium task just like the `search` tool. However, it bypasses DuckDuckGo and navigates directly to the requested URL.

Because it uses Chromium, the page fully evaluates React and Vue lifecycle hooks before the inner text is converted into Markdown and returned to the LLM context flow.

## Use Cases

If you are working with a completely new library, you can feed exactly the page you want directly to the model.

```md
User: "Can you write an initial setup script based on the instructions at https://bun.sh/docs/installation ?"
Agent: [Invokes fetch("https://bun.sh/docs/installation")]
Agent: "According to the official Bun documentation I just read, the command is..."
```
