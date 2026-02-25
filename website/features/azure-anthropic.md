# ☁️ Azure Anthropic & Token Caching

This fork provides native integration for Azure OpenAI/Foundry Anthropic deployments and real-time visualization of Anthropic's **Prompt Caching** (Beta).

## Native Azure Support

Standard Anthropic providers in Zed often fail with `404 DeploymentNotFound` because they passdated version strings (e.g., `claude-3-5-sonnet-20241022`) that Azure strictly rejects.

We modified `crates/anthropic/src/anthropic.rs` to intercept model resolution. When a custom `api_url` is detected, Zed automatically forwards the exact `serde_name` string you provided in settings, ensuring your Azure deployment name is matched perfectly.

## Prompt Caching UI

For massive context windows (200k+ tokens), sending the entire codebase on every turn is slow and expensive. Anthropic's **Prompt Caching** stores prefixes on their servers for roughly 5 minutes.

In this build, the `show_turn_stats` visualization is **enabled by default**.

<ZedChat>
  <template #assistant>
    (Turn Stats displayed above response)
    >> 250,000 tokens cached
    >> 1,200 tokens input
    >> 450 tokens output
  </template>
</ZedChat>

### Benefits
- **Persistent ROI**: Instantly see exactly how many tokens were pulled from the server cache.
- **Low Latency**: Responses start in milliseconds instead of multi-second wait times.
- **Enterprise Ready**: Seamlessly works with Azure AD and managed identity headers.
