# 🧠 Tool: `@recall`

The `@recall` tool is the retrieval engine for the SQLite long-term memory system, allowing the agent to pull past context dynamically based on search queries or categories.

## Technical Implementation

The agent invokes `@recall` to execute a `SELECT` query against the project's `memories` table. Unlike a global prompt injection, this tool allows for **selective retrieval**, keeping the prompt window clean while providing the necessary context.

The LLM can provide:
- `query`: A string for simple fuzzy matching.
- `category`: A filter to narrow down results (e.g., searching only in `Architecture`).
- `limit`: Controls how many facts are returned (defaults to the 5 most recent).

## Usage Example

<ZedChat>
  <template #user>
    How did we implement auth in this project?
  </template>
  <template #assistant>
    Searching project history for authentication patterns...
  </template>
  <template #output>
    @recall({ query: "authentication", category: "Architecture" })
    >> Found: "Base auth uses JWT with crates/core/src/auth.rs."
  </template>
</ZedChat>

## Advanced Usage
If the agent is facing a recurring compiler error, it will often invoke recall with `category: "Issues"` to see if a similar problem was solved and recorded via `@remember` in a previous session.
