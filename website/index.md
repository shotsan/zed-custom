---
# https://vitepress.dev/reference/default-theme-home-page---
layout: home

hero:
  name: "R-D Code"
  text: "The High-Performance AI Agent Fork"
  tagline: "Custom features for Zed including Long-Term Memory, LSP-based Context, and Headless Searching."
  image:
    src: /logo-animated.svg
    alt: R-D Code AI Logo
  actions:
    - theme: brand
      text: Features
      link: /features/memory
    - theme: alt
      text: GitHub
      link: https://github.com/shotsan/zed-custom

features:
  - title: 📚 Skill Library
    details: A modular prompt management system. Save refined prompts as "Skills" and hot-swap AI expertise (expert coding, auditing, architecture) instantly.
    link: /features/skill-library
  - title: 🧠 Long-Term Memory
    details: Persistent SQLite database that stores project context across restarts. No more context amnesia.
    link: /features/memory
  - title: 📉 Auto Context Compression
    details: Automatically condenses your long-running conversation tasks to keep your requests well under 200k model limits.
    link: /features/context-compression
  - title: ⚡ LSP Symbol Search
    details: Native Language Server Protocol integration for 100% accurate, type-aware code context.
    link: /features/lsp
  - title: 🌐 Headless Chromium
    details: Built-in chromiumoxide engine for real-time web search and JavaScript-heavy documentation.
    link: /features/search
  - title: 🧪 Deep Research Tool
    details: Autonomous, multi-stage recursive research engine. It identifies information gaps and performs follow-up searches to synthesize reports on complex topics.
    link: /features/tools/deep-research
  - title: ☁️ Azure Anthropic
    details: Native support for Azure deployments and real-time Token Caching (Prompt Caching) visualization.
    link: /features/azure-anthropic
  - title: 📝 Hybrid Skill System
    details: Merge project-scoped `.rules` files with your personal Global Skill Library. Instant expertise (security, performance, styling) at the tip of a slash command.
    link: /features/skill-library
  - title: 🔍 Custom Search Endpoint
    details: Mount any search endpoint (Elasticsearch, Algolia) as a native LLM tool call. The model can autonomously query your internal knowledge base and logs to solve issues. Invoke using <code>/custom-search</code>.
    link: /features/tools/custom-search
  - title: 🕵️ Full Message Privacy
    details: Audit every outgoing request to the LLM. No hidden telemetry. Total transparency on what data leaves your machine.
    link: /features/message-interception

---

> [!IMPORTANT]
> **Unofficial Fork**: This is a custom, unofficial fork of [Zed](https://github.com/zed-industries/zed) and is **not** associated with, endorsed by, or affiliated with [Zed Industries](https://zed.dev).
>
> **Licensing**: The enhancements in this fork are provided under the [MIT License](https://github.com/shotsan/zed-custom/blob/main/LICENSE-MIT). The original Zed source code follows its respective GPL/AGPL/Apache licenses.
---
