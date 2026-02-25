# 🧠 Tool: `@recall`

The `@recall` tool is the retrieval engine for the SQLite long-term memory system, allowing the agent to pull past context dynamically based on search queries or categories.

## Technical Implementation

The agent invokes `@recall` to execute a `SELECT` query against the project's `memories` table. Unlike a global prompt injection, this tool allows for **selective retrieval**, keeping the prompt window clean while providing the necessary context.

The LLM can provide:
- `query`: A string for simple fuzzy matching.
- `category`: A filter to narrow down results (e.g., searching only in `Architecture`).
- `limit`: Controls how many facts are returned (defaults to the 5 most recent).

## Usage Examples

### Example 1: Retrieval for Implementation Alignment
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

### Example 2: Debugging Recurring Issues
<ZedChat>
  <template #user>
    I'm getting a strange generic error in the `axum` handler. Have we seen this before?
  </template>
  <template #assistant>
    Searching project history for archived issue resolutions...
  </template>
  <template #output>
    @recall({ query: "axum error", category: "Issues" })
    >> Found: "Axum generic error fixed by adding `Send` bounds to the state type."
  </template>
</ZedChat>

## Workflow Impact
- **Deep Search**: Instantly filters through thousands of historical facts to find the one relevant to your current task.
- **Consistency**: Ensures the agent provides answers that align with *past* decisions rather than hallucinating new patterns.
- **Error Reduction**: Proactively retrieves "Lessons Learned" from the `Issues` category to prevent repeating past mistakes.
