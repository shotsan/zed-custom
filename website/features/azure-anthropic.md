# ☁️ Azure Anthropic & Token Caching

The IDE provides native integration for Azure OpenAI and Azure AI Foundry Anthropic deployments, along with real-time visualization of Anthropic's **Prompt Caching** performance.

---

## Native Azure Support

Connecting to Anthropic models via Azure requires precise model ID matching and endpoint routing. The IDE handles these specialized deployments out of the box, ensuring that custom deployment names are resolved correctly without requiring manual version string suffixes.

### Configuration Example

To use your Azure-hosted Anthropic deployment, configure your `settings.json` with the appropriate `api_url` and deployment names:

```json
{
  "language_models": {
    "anthropic": {
      "api_url": "https://YOUR_RESOURCE_NAME.services.ai.azure.com/anthropic",
      "available_models": [
        {
          "name": "YOUR_DEPLOYMENT_NAME",
          "display_name": "Azure Claude 3.5 Sonnet",
          "max_tokens": 200000
        }
      ]
    }
  }
}
```

---

## Prompt Caching UI

For massive context windows (200k+ tokens), processing the entire project on every turn can be slow. Anthropic's **Prompt Caching** stores frequently used project data on the server for immediate reuse.

The IDE displays these statistics in every message to provide full visibility into your "Cache ROI."

### Understanding the Statistics

- 🟢 **[X]k cached (Already Memorized)**: This is the code or documentation the model already "knows" from earlier in the conversation. Because the model doesn't have to re-read this data, it responds nearly instantly and costs significantly less.
- 🔵 **+[X]k saved (Newly Pinned)**: This is new information the model just read for the first time (like a newly opened file). The system is now "pinning" this to the model's high-speed memory so that it becomes "Cached" (Green) on your very next message.

---

## 🏗️ Three-Layer Caching Architecture

Instead of flagging data randomly, the IDE uses a multi-layered strategy to ensure maximum hit rates in every turn:

1.  **Base Layer (The Foundation)**: We always cache your project-specific rules, system prompts, and tool definitions. These are the core instructions the model needs for every single message.
2.  **Continuity Layer (Historical Context)**: We cache the very last message in your history. This essentially "locks in" the entire historical conversation prefix, ensuring the model doesn't have to re-read your old logs.
3.  **Active Layer (Task Processing)**: We cache your current message. This is critical for **Tool Use**. If the Assistant has to run a search or read a file *before* it answers you, the second part of that "brainstorming" cycle happens instantly.

### Technical Thresholds
- **Activation**: Caching is a server-side feature that typically kicks in once your prompt exceeds **1024 to 2048 tokens**. Conversations falling under this size will not show caching labels until the thread grows beyond this threshold crossed.
- **Persistence**: These cache entries live for approximately **5 minutes**. Frequent interactions in code-heavy threads will result in near 100% cache hit rates.

---

## Workflow Benefits

- **Enterprise Compliance**: Securely use high-performance models within managed Azure infrastructure.
- **Zero-Wait Context**: Massive projects that usually take 10s to process now respond in <1s.
- **Cost Optimization**: Reduce input costs by up to 90% for the "static" portions of your codebase.
