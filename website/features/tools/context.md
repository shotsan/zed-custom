# ⚡ Tool: `context`

The `context` tool is the interface between the AI agent and your project's **Language Server Protocol (LSP)**. It provides 100% accurate, compiler-verified code definitions.

## Technical Implementation

Standard Zed utilizes a background regex-based semantic indexer. This fork disables that subsystem and replaces it with `crates/agent/src/tools/context_tool.rs`. 

When the agent needs to understand a symbol, it forwards the request directly to the active server (`rust-analyzer`, `pyright`, etc.). This ensures the agent is reading the **actual resolved AST** of your project, not just a text-based guess.

## Usage Example

<ZedChat>
  <template #user>
    Where is the `spawn` method defined in the GPUI crate?
  </template>
  <template #assistant>
    Querying LSP for `spawn` definition...
  </template>
  <template #output>
    @context({ query: "spawn" })
    >> Found: crates/gpui/src/executor.rs:L112
  </template>
</ZedChat>

## Key Advantages
- **Type Sensitivity**: Distinguishes between local variables, types, and traits with the same name.
- **Zero Hallucination**: If the code won't compile because a symbol is missing, the agent will see that exact error via the LSP rather than guessing a location.
