---
# https://vitepress.dev/reference/default-theme-home-page---
layout: home

hero:
  name: "R-D Code"
  text: "The High-Performance AI Agentic IDE"
  tagline: "Long-term memory, LSP-aware context, deep research, and Azure model support — built into a fast, transparent coding environment."
  image:
    src: /logo-animated.svg
    alt: R-D Code AI Logo
  actions:
    - theme: brand
      text: Features
      link: /features/memory
    - theme: alt
      text: GitHub
      link: https://github.com/shotsan/agentic-ide

features:
  - title: 🧠 Long-Term Memory
    details: Persistent SQLite store for project context. Survives restarts — no context amnesia.
    link: /features/memory
  - title: 📚 Skill Library
    details: Save prompts as reusable Skills and hot-swap AI expertise (coding, auditing, architecture) with a slash command.
    link: /features/skill-library
  - title: 📉 Auto Context Compression
    details: Automatically condenses long conversations to stay within model context limits.
    link: /features/context-compression
  - title: ⚡ LSP Symbol Search
    details: Native Language Server Protocol integration for accurate, type-aware code context.
    link: /features/lsp
  - title: 🔎 Quick Web Search
    details: DuckDuckGo HTML scraper for fast fact-checking and automated link retrieval.
    link: /features/search
  - title: 🧪 Deep Research Tool
    details: Multi-stage recursive research engine that identifies gaps, runs follow-up searches, and synthesizes reports autonomously.
    link: /features/tools/deep-research
  - title: ☁️ Azure Anthropic
    details: Native Azure Anthropic deployment support with real-time Prompt Caching visualization.
    link: /features/azure-anthropic
  - title: 🤖 Azure OpenAI
    details: Connect any Azure OpenAI deployment via Chat Completions or Responses API. Configure endpoint, deployment name, and API version in settings.
    link: /features/azure-openai
  - title: 🔍 Custom Search Endpoint
    details: Mount any search endpoint (Elasticsearch, Algolia) as a native LLM tool. The model queries your internal knowledge base autonomously via <code>/custom-search</code>.
    link: /features/tools/custom-search
  - title: 🕵️ Full Message Privacy
    details: Every outgoing LLM request is auditable. No hidden telemetry — total transparency on what leaves your machine.
    link: /features/message-interception

---

> [!IMPORTANT]
> **Unofficial Fork**: This is a custom, unofficial fork and is **not** associated with, endorsed by, or affiliated with the original developers.
>
> **Licensing**: Enhancements are provided under the [MIT License](https://github.com/shotsan/agentic-ide/blob/main/LICENSE-MIT). The original source code follows its respective GPL/AGPL/Apache licenses.
---
