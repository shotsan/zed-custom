# 🧠 Tool: `@recall`

Query historical project context from the SQLite database.

## Technical Details
- **Internal File**: `crates/agent/src/tools/memory_tools.rs`
- **Logic**: Executes a `SELECT` query with optional `query` and `category` filters.

## Usage Example

<ZedChat>
  <template #user>
    How did we implement auth in this project?
  </template>
  <template #assistant>
    Searching project memories for "authentication"...
  </template>
  <template #output>
    @recall({ query: "authentication", category: "Architecture" })
    >> Found: "Base auth uses JWT with crates/core/src/auth.rs."
  </template>
</ZedChat>

> [!TIP]
> Use specific keywords to filter through months of development history instantly.
