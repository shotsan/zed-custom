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

## Workflow Impact
- **Enterprise Compliance**: Securely use LLMs within Azure's managed infrastructure and VPCs.
- **Zero-Wait Context**: Massive projects that usually take 10s to process now respond in <1s due to prefix caching.
- **Dramatic Cost Savings**: Pay up to 90% less for the "static" project context that doesn't change between turns.
- **Perfect Routing**: Eliminates the "404 Model Not Found" errors by matching exact Azure deployment names.
