# ⚡ Tool: `context`

Query the active Language Server (LSP) for 100% accurate, type-aware code context.

## Technical Details
- **Internal File**: `crates/agent/src/tools/context_tool.rs`
- **Logic**: Forwards symbol requests to the native LSP (`rust-analyzer`, `pyright`, etc.).

## Usage Example

<ZedChat>
  <template #user>
    Where is the `spawn` method defined?
  </template>
  <template #assistant>
    Locating `spawn` definition...
  </template>
  <template #output>
    @context({ query: "spawn" })
    >> Found: crates/gpui/src/executor.rs:L112
  </template>
</ZedChat>

### Advantages
- **No Indexing**: Uses existing LSP memory.
- **Precision**: Handles macros and trait implementations accurately.
