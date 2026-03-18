# 🔍 Custom Search Endpoint

Registering a Custom Search Endpoint transforms your internal APIs (e.g., Elasticsearch, Algolia, or any generic HTTPS POST search API) into a **native tool call** directly exposed to your configured LLM model. 

This enables you to interact with your data in two ways:

---

## Feature 1: Agent Tool (`custom_search`)

By saving your endpoint, the `custom_search` tool is automatically registered in your LLM's system prompt. This gives the **configured AI model** the ability to autonomously execute searches against your logs, docs, and internal systems without you having to manually query them.

### How it works

When the agent is working on a task that requires context you haven't provided yet, the model will dynamically invoke the `custom_search` tool call. You will see the tool execution appear in the Agent Panel as a collapsible card, just like any other native Zed tool (`read_file`, `terminal`, etc.).

### When the model uses it

The model will use this tool when you ask things like:
- "Find all error logs from the last deployment"
- "Search my index for documents about user authentication"
- "What does our system contain about payment failures?"

### Permission

Like other network tools, `custom_search` will ask for your confirmation before running unless you have `always_allow_tool_actions` set to `true` in your agent settings.

---

## Feature 2: Slash Command (`/custom-search`)

The `/custom-search` slash command lets **you** manually trigger a query from the Assistant panel's message input. It inserts the raw JSON results directly into the conversation as context for the model.

### How to use it

In the Assistant panel (old thread view), type:

```
/custom-search your query here
```

For example:

```
/custom-search level:error AND service:payments
```

The results are fetched immediately and inserted into the thread as a formatted JSON code block, which the model can then read and reason about.

### When to use the slash command vs. the agent tool

| | `/custom-search` slash command | `custom_search` agent tool |
|---|---|---|
| **Triggered by** | You, manually | The AI model, automatically |
| **Where** | Assistant panel message input | Agent panel (autonomous agent) |
| **Output** | JSON inserted into thread | Tool result shown as a card in the Agent panel |
| **Use case** | When you want to provide search data as context yourself | When you want the agent to search your endpoint as part of a larger task |

---

## Configuration

Both the slash command and the agent tool read from the same configuration block inside your `settings.json`. 

> [!NOTE]
> **First-Time Installation:** This setting may get reset when you install Zed Custom for the first time. If you want to use the default `doc-search.dev` capabilities, note that you will need to manually add the current search URL and port: `http://52.179.183.195:9200`.

You can add this URL using two methods:

### Method 1: Using the Assistant Panel UI (Recommended)

Zed Custom includes a direct UI shortcut for adding your search endpoint so you don't even have to write JSON manually!

1. Open the **Assistant Panel** (the chat panel).
2. Click the **Gear icon** (Settings) at the top right of the Assistant panel to open the underlying configuration modal.
3. Scroll down past your Agent Profiles to find a section titled **"Custom Search"** (with the description *"Configure your own search endpoint for agent tools."*).
4. Click the small **Arrow (Open)** icon next to the title. This is wired up directly to Zed's settings editor and will instantly open your `settings.json` file and drop your cursor precisely where the custom search configuration belongs!

### Method 2: Manually in `settings.json`

If you want to add it manually without clicking through the UI, open your Zed `settings.json` (using `Cmd-,`), and look for the `"agent"` object. Add the `"custom_search"` key inside of it like this:

```json
{
  "agent": {
    "custom_search": {
      // Add your actual URL here (e.g., Elasticsearch, Algolia endpoint)
      "endpoint_url": "http://52.179.183.195:9200",
      
      // Optional API Key for Authorization
      "api_key": "YOUR_API_KEY_HERE"
    }
  }
}
```

| Setting | Required | Description |
|---|---|---|
| `endpoint_url` | **Yes** | Full HTTP URL to your search endpoint (e.g., Elasticsearch `_search` endpoint). |
| `api_key` | No | API key sent as `Authorization: ApiKey <key>`. Omit if authentication is not required. |

> **Both features stop working and return an error if `endpoint_url` is not set.** The `api_key` is optional.

---

## Example: Using Both Together

A typical workflow might combine both: you use `/custom-search` to do a quick investigation first, then ask the agent to take action on the results — at which point it may call `custom_search` on its own for follow-up queries.

```
/custom-search service:checkout AND status:500 AND @timestamp:[now-1h TO now]
```

Then in a follow-up message:

```
Based on those results, investigate why the checkout service is failing and suggest a fix.
```

The agent will understand the context you provided via `/custom-search` and may issue additional `custom_search` calls autonomously as it investigates.
