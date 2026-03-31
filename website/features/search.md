# 🌐 Headless Chromium Search

The ultimate AI assistant shouldn't just know your code; it should be able to read external documentation. 

This custom fork integrates a full headless Chrome engine (`chromiumoxide`), seamlessly hooked into a new `/search` panel command.

## The `/search` Command

By typing `/search <query>` into the Assistant Panel, you trigger an asynchronous background task that boots a headless Chromium instance, performs a Search (via DuckDuckGo or Google), navigates to the top URLs, and executes JavaScript to render SPA applications (like React or Vue docs).

Once the page is rendered, Zed extracts the Markdown payload and synthesizes a perfect answer for you, right inline with your current code.

![Search Tool Demo](/demo-search.gif)

## Visual Walkthrough

### Example 1: Resolving Library Knowledge Cutoffs
<ZedChat>
  <template #user>
    Write a simple server using the latest `axum` version. I'm not sure if the API changed in 0.7.
  </template>
  <template #assistant>
    Searching for `axum 0.7 release notes` to verify API changes...
  </template>
  <template #output>
    @search({ query: "axum 0.7 breaking changes" })
    >> Results: docs.rs/axum/latest/..., entries found.
    @fetch({ url: "https://docs.rs/axum/latest/axum/index.html" })
    >> Extracting Axum 0.7 routing syntax...
  </template>
</ZedChat>

### Example 2: Comparative Documentation Research
<ZedChat>
  <template #user>
    Should I use `tokio::sync::mpsc` or `crossbeam_channel` for a high-throughput UI event loop?
  </template>
  <template #assistant>
    Comparing performance benchmarks and async compatibility...
  </template>
  <template #output>
    @search({ query: "tokio mpsc vs crossbeam performance benchmarks" })
    >> Found benchmark analysis on Reddit and GitHub Discussions.
  </template>
</ZedChat>

## Workflow Impact
- **End-to-End Solutions**: The agent no longer says "I don't know the latest API." It finds it, reads it, and implements it.
- **Fact Verification**: Instantly cross-reference agent suggestions against official documentation without leaving the editor.
- **Context Expansion**: Pull in external examples, RFCs, and GitHub issues to solve obscure architectural problems.
- **Accurate Dependencies**: The agent can verify the exact latest version strings for your `Cargo.toml`.
- **Search Engine Flexibility**: Choose between Google and DuckDuckGo to optimize for the richest search results.
