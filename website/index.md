---
# https://vitepress.dev/reference/default-theme-home-page---
layout: home

hero:
  name: "The Zed AI Fork"
  text: "10x Developer Velocity."
  tagline: Limitless Headless Browsing. Deep SQLite Memory. Pure Speed.
  image:
    src: /logo.png
    alt: Zed Custom Fork Logo
  actions:
    - theme: brand
      text: Explore Features
      link: /features/memory
    - theme: alt
      text: View Source
      link: https://github.com/shotsan/zed-custom

features:
  - title: 🧠 Persistent Local DB
    details: Standard IDEs lose context on restart. We automatically persist architectural decisions, patterns, and bugs in a lightweight SQLite database using `@remember` and `@recall`.
  - title: 🌐 Headless Chromium Search
    details: Hit an obscure compiler bug? Type `/search` in the chat panel to instantly bypass bot protection and extract JavaScript-rendered documentation without ever leaving the IDE.
  - title: ☁️ Azure Anthropic Caching
    details: Stop waiting for 200k context requests. Natively handles Azure deployment string endpoints and exposes token caching (`X saved`, `X cached`) visualizations directly above the chat box.
  - title: ⚡️ LSP Powered Context
    details: We disabled the CPU-destroying background Regex indexers. Our AI agent queries your language server (`rust-analyzer`, `pyright`) natively for perfectly typed, hallucination-free context.
  - title: 📝 Custom System Prompts & Rules
    details: Project-scoped `.rules` files drop directly into the agent's subconscious prompt, perfectly aligning its persona and coding standards to your exact repository requirements.
  - title: 🕵️ Full Message Privacy
    details: Don't guess what the AI knows. Our open source engine allows for full message interception, meaning you can audit the exact JSON payload and telemetry sent over the wire.
---
