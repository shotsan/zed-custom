---
# https://vitepress.dev/reference/default-theme-home-page---
layout: home

hero:
  name: "Zed Custom"
  text: "The High-Performance AI Agent Fork"
  tagline: "Custom features for Zed including Long-Term Memory, LSP-based Context, and Headless Searching."
  image:
    src: /logo-animated.svg
    alt: Zed Custom AI Logo
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
  - title: 🧠 Long-Term Memory
    details: Persistent SQLite database that stores project context across restarts. No more context amnesia.
  - title: ⚡ LSP Symbol Search
    details: Native Language Server Protocol integration for 100% accurate, type-aware code context.
  - title: 🌐 Headless Chromium
    details: Built-in chromiumoxide engine for real-time web search and JavaScript-heavy documentation.
  - title: ☁️ Azure Anthropic
    details: Native support for Azure deployments and real-time Token Caching (Prompt Caching) visualization.
  - title: 📝 Custom System Prompts & Rules
    details: Project-scoped `.rules` files drop directly into the agent's subconscious prompt, perfectly aligning its persona and coding standards to your exact repository requirements.
  - title: 🔍 Elasticsearch Integration
    details: Query your Elasticsearch indices directly from the agent or via the <code>/elastic</code> slash command. The AI can autonomously search logs, documents, and records as part of any task.
    link: /features/tools/elastic-search
  - title: 🕵️ Full Message Privacy
    details: Audit every outgoing request to the LLM. No hidden telemetry. Total transparency on what data leaves your machine.

---

> [!IMPORTANT]
> **Unofficial Fork**: This is a custom, unofficial fork of [Zed](https://github.com/zed-industries/zed) and is **not** associated with, endorsed by, or affiliated with [Zed Industries](https://zed.dev).
>
> **Licensing**: The enhancements in this fork are provided under the [MIT License](https://github.com/shotsan/zed-custom/blob/main/LICENSE-MIT). The original Zed source code follows its respective GPL/AGPL/Apache licenses.
---
