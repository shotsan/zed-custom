# 🧠 Tool: `@remember`

The `@remember` tool allows the AI agent to persist critical project context into a long-term SQLite database, ensuring important decisions or patterns are not lost between sessions.

## Technical Implementation

When the agent invokes `@remember`, the system executes an `INSERT OR REPLACE` SQL statement against the local `memories` table (located in `~/.local/share/zed/`). 

The implementation in `crates/agent/src/tools/memory_tools.rs` enforces categorization into one of five schemas:
1.  **Architecture**: structural rules.
2.  **Patterns**: code styles.
3.  **Issues**: bug resolutions.
4.  **Procedures**: build/deploy steps.
5.  **Notes**: general facts.

## Usage Example

<ZedChat>
  <template #user>
    Note that this project uses Rust 1.76 specifically for this workspace.
  </template>
  <template #assistant>
    Persisting tool-chain constraint to memory...
  </template>
  <template #output>
    @remember({
      category: "Procedures",
      content: "Workspace strictly uses Rust 1.76."
    })
  </template>
</ZedChat>

> [!IMPORTANT]
> Because it uses `INSERT OR REPLACE`, the agent can update existing memories by targeting the same category and refining the content, allowing the project's knowledge base to evolve naturally.
