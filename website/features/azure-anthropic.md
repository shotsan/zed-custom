# ☁️ Azure Anthropic & Token Caching

## 1. Zero-Friction Azure Support

This build provides native support for Azure's Anthropic API deployments, including real-time visualization of Anthropic's **Prompt Caching** (Beta).

## Cache Visualization

When you send a message, the chat UI displays exactly how many tokens were cached and saved, drastically reducing both latency and cost for large context windows.

<ZedChat>
  <template #assistant>
    (Turn Stats displayed above response)
    >> 250,000 tokens cached
    >> 1,200 tokens input
    >> 450 tokens output
  </template>
</ZedChat>

## Configuration

Unlike standard Anthropic providers, Azure requires a specific endpoint format. We handle this transformation internally so you only need to provide your Base URL and Management Key.

### Key Benefits
- **Persistent Cache**: Keeps your project context "warm" across turns.
- **Cost Reduction**: Up to 90% cheaper for repeated massive prompts.
- **Low Latency**: Responses begin instantly even with 200k+ tokens of context.
