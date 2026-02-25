# ⚡️ LSP Symbol Context

We replaced background Regex indexing with native Language Server Protocol (LSP) integration. This provides 100% accurate, type-aware context for the AI agent.

## How it works

The agent uses the `context` tool to query your active language server (`rust-analyzer`, `gopls`, `pyright`, etc.) for precise symbol locations and definitions.

<ZedChat>
  <template #user>
    Where is the `UserSession` struct defined and used?
  </template>
  <template #assistant>
    Querying LSP for `UserSession` references...
  </template>
  <template #output>
    @context({ query: "UserSession" })
    >> Definition: crates/core/src/auth.rs:45
    >> References: [main.rs:12, api/v1/user.rs:89, ...]
  </template>
</ZedChat>

## Why this is better

- **Zero CPU Waste**: No massive background threading for string-matching indexes.
- **Type Awareness**: The agent understands the difference between a variable and a type with the same name.
- **Reliable Refs**: If the LSP can find it, the agent can find it.

This guarantees the model isn't hallucinating references—it is strictly reading from the active compiler state.
