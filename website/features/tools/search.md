# 🌐 Tool: `search`

The `search` tool is an incredibly powerful addition that grants the agent access to real-time information on the open internet using a custom headless Chromium engine.

## Technical Details

**Internal File:** `crates/agent/src/tools/web_search_tool.rs`

When the user types `/search <query>` into the chat panel, it immediately bypasses standard LLM context and invokes the specific DuckDuckGo search API underneath the Chromium headless layer.

Because it uses `chromiumoxide` to execute this, it can bypass standard bot protection, render complete JavaScript Single Page Applications (SPAs), and extract the pure Markdown representation of the page.

## Use Cases

1. **Reading Modern Docs**: Scraping new libraries or frameworks that were released after the LLM's static training cutoff date.
2. **Finding Solutions**: Taking an obscure compiler error and letting the agent silently search StackOverflow to find identical issues.

```md
User: "/search What is the latest API shape for VuePress?"
Agent: [Invokes search, scrapes 3 links, distills the answer]
```
