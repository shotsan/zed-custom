# ☁️ Azure OpenAI

Zed Custom supports Azure OpenAI as a first-class language model provider. You can connect any model deployed in your Azure AI Foundry resource and use it as the default model for the agent, including full tool use.

---

## How It Works

The provider integrates with two Azure OpenAI APIs depending on how you configure your deployment:

- **Chat Completions API** (`/openai/deployments/<deployment>/chat/completions`) — used for standard chat models such as GPT-4o. This is the default mode.
- **Responses API** (`/openai/responses`) — used for models that do not support Chat Completions (e.g., `o-series` reasoning models). The provider automatically routes to this endpoint when you disable the **Chat Model** capability on the deployment.

You do not need to manually specify which endpoint to use. The `capabilities.chat_completions` flag in your settings controls the routing.

---

## Configuration via the UI

Open the Agent panel, click the model selector, and choose **Azure OpenAI**. The configuration view will ask for:

| Field | Description |
|---|---|
| **API URL** | Your Azure Cognitive Services resource endpoint, e.g. `https://<resource>.cognitiveservices.azure.com/` |
| **API Version** | The Azure OpenAI API version string, e.g. `2025-04-01-preview` |
| **API Key** | Your Azure resource API key |
| **Deployment Name** | The exact name of your deployment as it appears in Azure AI Foundry |
| **Max Tokens** | The context window size of the deployed model |
| **Max Output Tokens** | The maximum number of tokens the model may generate per response (optional) |

After saving, the deployment is stored in your `settings.json` under `language_models.azure_openai`.

---

## Configuration via `settings.json`

You can configure the provider directly in your user settings file for full control. The settings key is `language_models.azure_openai`.

### Minimal example (Chat Completions model)

```json
{
  "language_models": {
    "azure_openai": {
      "api_url": "https://<YOUR_RESOURCE_NAME>.cognitiveservices.azure.com/",
      "api_version": "2025-04-01-preview",
      "available_models": [
        {
          "name": "<YOUR_DEPLOYMENT_NAME>",
          "display_name": "GPT-4o (Azure)",
          "max_tokens": 128000,
          "max_output_tokens": 16384
        }
      ]
    }
  }
}
```

### Reasoning model example (Responses API)

For `o-series` or other models that do not support Chat Completions, set `chat_completions` to `false`. The provider will automatically use the Responses API endpoint (`/openai/responses`) instead.

```json
{
  "language_models": {
    "azure_openai": {
      "api_url": "https://<YOUR_RESOURCE_NAME>.cognitiveservices.azure.com/",
      "api_version": "2025-04-01-preview",
      "available_models": [
        {
          "name": "<YOUR_DEPLOYMENT_NAME>",
          "display_name": "o3 (Azure)",
          "max_tokens": 200000,
          "max_output_tokens": 100000,
          "capabilities": {
            "chat_completions": false,
            "tool_use": true
          }
        }
      ]
    }
  }
}
```

---

## All Settings Fields

### `azure_openai` (top-level)

| Field | Type | Required | Description |
|---|---|---|---|
| `api_url` | string | **Yes** | Your Azure resource base URL. Must end in `/`. Example: `https://my-resource.cognitiveservices.azure.com/` |
| `api_version` | string | **Yes** | Azure API version query parameter. Example: `2025-04-01-preview` |
| `available_models` | array | **Yes** | List of deployments to expose in the model selector |

### Each entry in `available_models`

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | **Yes** | Your exact Azure deployment name. This is used verbatim in the URL path `/openai/deployments/<name>/`. It is **not** the model family name (e.g. do not use `gpt-4o` unless your deployment is literally named `gpt-4o`). |
| `display_name` | string | No | Label shown in the UI model selector. Defaults to `name` if omitted. |
| `max_tokens` | number | **Yes** | Context window size in tokens. Used for token counting. |
| `max_output_tokens` | number | No | Maximum tokens the model produces per response. |
| `max_completion_tokens` | number | No | Alternate token limit field (reserved). |
| `reasoning_effort` | string | No | One of `"minimal"`, `"low"`, `"medium"`, `"high"`. Passed to the API for reasoning models that support it. |
| `capabilities.chat_completions` | boolean | No | Defaults to `true`. Set to `false` to route requests through the Responses API instead of Chat Completions. |
| `capabilities.tool_use` | boolean | No | Defaults to `true`. Set to `false` to disable tool definitions from being sent in requests. |

---

## API Key

The API key can be provided in two ways:

1. **Via the UI** — Enter it in the configuration view. It is stored securely in the system keychain, keyed by your `api_url`.
2. **Via environment variable** — Set `AZURE_OPENAI_API_KEY` before launching Zed Custom. The app reads this on startup and you do not need to enter anything in the UI.

> [!IMPORTANT]
> The API key is stored per-resource-URL. If you change your `api_url`, you will need to re-enter the API key in the configuration view.

---

## How the URL Is Constructed

Understanding how the provider builds the request URL helps diagnose connection issues.

### Chat Completions path

Given:
- `api_url` = `https://my-resource.cognitiveservices.azure.com/`
- `api_version` = `2025-04-01-preview`
- `name` (deployment) = `my-gpt4o`

The provider constructs:
```
https://my-resource.cognitiveservices.azure.com/openai/deployments/my-gpt4o/chat/completions?api-version=2025-04-01-preview
```

The `/openai/deployments/<name>` segment is appended automatically. You do **not** need to include it in `api_url`.

### Responses API path

For models with `chat_completions: false`, the provider constructs:
```
https://my-resource.cognitiveservices.azure.com/openai/responses?api-version=2025-04-01-preview
```

Note that the Responses API is **resource-scoped**, not deployment-scoped. The deployment name is passed inside the request body as the `model` field.

---

## Common Pitfall: Deployment Name vs. Model Name

> [!WARNING]
> The `name` field must be your **Azure Deployment Name**, not the underlying model family name.

When you create a deployment in Azure AI Foundry, you assign it a name. That name is what Azure expects in the URL or request body. If your deployment is named `prod-gpt4o` but you set `"name": "gpt-4o"`, Azure will return a `404 DeploymentNotFound` error because it will search your resource for a deployment literally called `gpt-4o`.

**Correct:**
```json
{ "name": "prod-gpt4o" }
```

**Incorrect (causes 404):**
```json
{ "name": "gpt-4o" }
```

---

## Setting as Default Agent Model

After configuring, the model will appear in the model selector. To set it as the default in `settings.json`:

```json
{
  "agent": {
    "default_model": {
      "provider": "azure_openai",
      "model": "<YOUR_DEPLOYMENT_NAME>"
    }
  }
}
```

The `model` value must match the `name` field of the entry in `available_models`.
