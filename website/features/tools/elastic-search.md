# 🔍 Elasticsearch Integration

Zed has **two separate but complementary** ways to interact with Elasticsearch. Understanding both is important — they serve different purposes and operate in different parts of the UI.

---

## Feature 1: Agent Tool (`elastic_search`)

The agent tool gives the **AI model itself** the ability to query Elasticsearch. The model decides autonomously when to call it based on the conversation context.

### How it works

When the agent is working on a task that requires searching your Elasticsearch data (e.g. finding logs, documents, or records), it will call the `elastic_search` tool automatically. You will see the tool call appear in the Agent Panel as a collapsible card, just like any other tool use (`read_file`, `terminal`, etc.).

### When the model uses it

The model will use this tool when you ask things like:
- "Find all error logs from the last deployment"
- "Search my Elasticsearch index for documents about user authentication"
- "What does our index contain about payment failures?"

### Permission

Like other network tools, `elastic_search` will ask for your confirmation before running unless you have `always_allow_tool_actions` set to `true` in your agent settings.

---

## Feature 2: Slash Command (`/elastic`)

The `/elastic` slash command lets **you** manually trigger an Elasticsearch query from the Assistant panel's message input. It inserts the raw JSON results directly into the conversation as context for the model.

### How to use it

In the Assistant panel (old thread view), type:

```
/elastic your query here
```

For example:

```
/elastic level:error AND service:payments
```

The results are fetched immediately and inserted into the thread as a formatted JSON code block, which the model can then read and reason about.

### When to use the slash command vs. the agent tool

| | `/elastic` slash command | `elastic_search` agent tool |
|---|---|---|
| **Triggered by** | You, manually | The AI model, automatically |
| **Where** | Assistant panel message input | Agent panel (autonomous agent) |
| **Output** | JSON inserted into thread | Tool result shown as a card in the Agent panel |
| **Use case** | When you want to provide Elasticsearch data as context yourself | When you want the agent to search Elasticsearch as part of a larger task |

---

## Configuration

Both the slash command and the agent tool read from the same configuration block in your `settings.json`:

```json
{
  "agent": {
    "elastic_search": {
      "endpoint_url": "https://your-elasticsearch-host:9200/your-index",
      "api_key": "your_api_key_here"
    }
  }
}
```

| Setting | Required | Description |
|---|---|---|
| `endpoint_url` | **Yes** | Full URL to your Elasticsearch index (including index name). Appends `/_search` automatically if not already present. |
| `api_key` | No | API key sent as `Authorization: ApiKey <key>`. Omit if your Elasticsearch does not require authentication. |

> **Both features stop working and return an error if `endpoint_url` is not set.** The `api_key` is optional for open clusters.

---

## Example: Using Both Together

A typical workflow might combine both: you use `/elastic` to do a quick investigation first, then ask the agent to take action on the results — at which point it may call `elastic_search` on its own for follow-up queries.

```
/elastic service:checkout AND status:500 AND @timestamp:[now-1h TO now]
```

Then in a follow-up message:

```
Based on those results, investigate why the checkout service is failing and suggest a fix.
```

The agent will understand the context you provided via `/elastic` and may issue additional `elastic_search` calls autonomously as it investigates.
