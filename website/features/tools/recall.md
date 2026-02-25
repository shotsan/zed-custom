# 🧠 Tool: `@recall`

The `recall` tool is the reading mechanism for the SQLite long-term memory system. It allows the agent to pull past context completely dynamically without you having to manually find and copy files into the prompt window.

## Technical Details

**Internal File:** `crates/agent/src/tools/memory_tools.rs`

When the agent uses the tool, it executes a `SELECT` query against the `memories` table. 

Parameters the LLM can provide:
- `query` (Optional string fuzzing)
- `category` (Optional filtering by Pattern, Issue, Architecture, etc.)
- `limit` (Defaults to grabbing the 5 most recently accessed memories)

## How it works

When a new session begins, the system prompt actively instructs the agent that it can retrieve historical context by using `recall`. If you ask a question about something you discussed weeks ago, the agent will query the database, read the results into the prompt window transparently, and use that returned context to format its response.

```md
User: "How did we implement the authentication layer again?"
Agent: [Invokes recall(query="authentication", category="Architecture")]
Agent: "Based on our past decisions, we used JSON Web Tokens with..."
```
