# 🌐 Headless Chromium Search

The ultimate AI assistant shouldn't just know your code; it should be able to read external documentation. 

This custom fork integrates a full headless Chrome engine (`chromiumoxide`), seamlessly hooked into a new `/search` panel command.

## The `/search` Command

By typing `/search <query>` into the Assistant Panel, you trigger an asynchronous background task that boots a headless Chromium instance, performs a DuckDuckGo search, navigates to the top URLs, and executes JavaScript to render SPA applications (like React or Vue docs).

Once the page is rendered, Zed extracts the Markdown payload and synthesizes a perfect answer for you, right inline with your current code.

![Search Tool Demo](/demo-search.gif)

### Key Advantages
- **No API Keys Needed**: Unlike third-party search APIs, this uses a local Chromium instance to scrape open web results for free.
- **JavaScript Rendering**: Traditional scrapers fail on modern documentation sites. Headless Chrome ensures even the heaviest React apps are fully rendered before extraction.
- **Deep Dives via `@fetch`**: You can explicitly command the agent to read specific pages by typing `@fetch https://example.com/docs`, embedding entire remote pages into your prompt window instantly.
