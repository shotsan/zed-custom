# 🧠 Tool: `@remember`

The `remember` tool gives the Zed AI Agent the ability to write permanent context directly to a SQLite database tied exclusively to your current workspace root.

## Technical Details

**Internal File:** `crates/agent/src/tools/memory_tools.rs`

When invoked, the tool executes an `INSERT OR REPLACE` SQL statement against your project's `memories` table. The LLM is forced to categorize the input into one of 5 distinct schemas:
1. `Architecture`
2. `Patterns`
3. `Issues`
4. `Procedures`
5. `Notes`

## How it's Used

Because the LLM is autonomous, it can choose to use the `remember` tool proactively when it solves a complex bug, or you can explicitly command it to memory.

```md
User: "Please remember that we use Rust 1.76 and strict Clippy warnings."
Agent: [Invokes remember(category="Constraints", content="Uses Rust 1.76...")]
```

> **Note:** The memory database is stored locally in `~/.local/share/zed/`. No context is sent to third-party databases for storage.
