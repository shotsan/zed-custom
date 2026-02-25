---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "Zed Custom"
  text: "The Ultimate AI Editor."
  tagline: A blisteringly fast code editor, now featuring persistent Long-Term AI Memory, headless React web search, and Azure Anthropic Caching built natively into the engine.
  actions:
    - theme: brand
      text: Explore Features
      link: /features/memory
    - theme: alt
      text: View on GitHub
      link: https://github.com/shotsan/zed-custom

features:
  - title: 🧠 Persistent Memory
    details: The AI agent natively stores patterns, architecture, and bugs in a local SQLite database across restarts using "@remember" and "@recall".
  - title: 🌐 Headless Chromium Search
    details: Type "/search" directly in the chat panel to instantly browse documentation and extract JavaScript-heavy website context without ever leaving the IDE.
  - title: ☁️ Azure Anthropic Native
    details: Natively handles Azure deployment string endpoints and exposes token caching visualizations by default to drastically reduce massive prompt latency.
  - title: ⚡️ LSP Powered Context
    details: Replaces CPU-heavy background regex indexing with completely silent, instant, type-aware language server protocol (LSP) integrations.
---
