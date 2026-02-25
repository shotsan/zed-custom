---
# https://vitepress.dev/reference/default-theme-home-page---
layout: home

hero:
  name: "Zed Custom"
  text: "The High-Performance AI Agent Fork"
  tagline: "Custom features for Zed including Long-Term Memory, LSP-based Context, and Headless Searching."
  actions:
    - theme: brand
      text: Features
      link: /features/memory
    - theme: alt
      text: GitHub
      link: https://github.com/shotsan/zed-custom

features:
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
  - title: 🕵️ Full Message Privacy
    details: Don't guess what the AI knows. Our open source engine allows for full message interception, meaning you can audit the exact JSON payload and telemetry sent over the wire.
---
