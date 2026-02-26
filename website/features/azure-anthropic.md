# ☁️ Azure Anthropic & Token Caching

This fork provides native integration for Azure OpenAI/Foundry Anthropic deployments and real-time visualization of Anthropic's **Prompt Caching** (Beta).

## Native Azure Support

Standard Anthropic providers in Zed often fail with `404 DeploymentNotFound` because they passdated version strings (e.g., `claude-3-5-sonnet-20241022`) that Azure strictly rejects.

We modified `crates/anthropic/src/anthropic.rs` to intercept model resolution. When a custom `api_url` is detected, Zed automatically forwards the exact `serde_name` string you provided in settings, ensuring your Azure deployment name is matched perfectly.

## Prompt Caching UI

For massive context windows (200k+ tokens), sending the entire codebase on every turn is slow and expensive. Anthropic's **Prompt Caching** stores prefixes on their servers for roughly 5 minutes.

In this build, the `show_turn_stats` visualization is **enabled by default**.

## Visual Walkthrough: Caching ROI

<ZedChat>
  <template #user>
    (First Turn: Sends 50 files)
    Summarize the overall architecture.
  </template>
  <template #assistant>
    >> 0 tokens cached
    >> 45,000 tokens input
    Generating summary...
  </template>
</ZedChat>

<ZedChat>
  <template #user>
    (Second Turn: Asks follow-up)
    Now find the specific logic for the auth handler.
  </template>
  <template #assistant>
    >> 45,200 tokens cached (ROI! ⚡️)
    >> 150 tokens input
    Locating auth handler...
  </template>
</ZedChat>

## Smart Caching Strategy

The caching mechanism is fully automated and optimized to maximize the 5-minute cache window provided by Anthropic. We implement a multi-layered approach to ensure your context remains "hot":

1.  **System Prompt Caching**: The base system prompt and injected tool definitions are permanently flagged for caching. This ensures that even in new threads, the foundational instructions are pre-cached.
2.  **The "Prefix Window" Strategy**: We strategically flag the **second-to-last message** in every turn. By doing this, we ensure that as the conversation grows, the entire history leading up to your latest prompt is stored as a reusable prefix on Anthropic's servers.
3.  **Interval Stabilization**: Caching is re-enforced every 15 messages and at the midpoint of long threads to prevent cache eviction during intense, long-running debugging sessions.

### Technical Thresholds
- **Activation**: Anthropic typically activates caching for prompts exceeding **1024 or 2048 tokens**. Small conversations will not show caching tokens until this threshold is crossed.
- **Persistence**: Cache entries live for approximately **5 minutes** on the server. Frequent turn-arounds in code-heavy threads will result in near 100% cache hit rates.

## Workflow Impact
- **Enterprise Compliance**: Securely use LLMs within Azure's managed infrastructure and VPCs.
- **Zero-Wait Context**: Massive projects that usually take 10s to process now respond in <1s due to prefix caching.
- **Dramatic Cost Savings**: Pay up to 90% less for the "static" project context that doesn't change between turns.
- **Perfect Routing**: Eliminates the "404 Model Not Found" errors by matching exact Azure deployment names.

### References & Source Code
- [`crates/agent/src/thread.rs`](file:///Users/sillydon/Desktop/zed/crates/agent/src/thread.rs) — Implements the message flagging and prefix logic.
- [`crates/language_models/src/provider/anthropic.rs`](file:///Users/sillydon/Desktop/zed/crates/language_models/src/provider/anthropic.rs) — Handles the `cache_control` headers.
- [`crates/agent_ui/src/acp/thread_view.rs`](file:///Users/sillydon/Desktop/zed/crates/agent_ui/src/acp/thread_view.rs) — Manages the green/blue token visualization.
