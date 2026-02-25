# 🧠 Tool: `@remember`

Persist architectural facts and patterns directly to the project's SQLite database.

## Technical Details
- **Internal File**: `crates/agent/src/tools/memory_tools.rs`
- **Logic**: Executes `INSERT OR REPLACE` into the `memories` table.

## Usage Example

<ZedChat>
  <template #user>
    Remember that this crate uses Rust 1.76 specifically for this workspace.
  </template>
  <template #assistant>
    Recording project constraint...
  </template>
  <template #output>
    @remember({ category: "Procedures", content: "Uses Rust 1.76 strictly." })
  </template>
</ZedChat>

> [!NOTE]
> Database is stored locally at `~/.local/share/zed/`. No context is sent to third-party databases.
