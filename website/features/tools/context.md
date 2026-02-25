# ⚡ Tool: `context`

The `context` tool is the interface between the AI agent and your project's **Language Server Protocol (LSP)**. It provides 100% accurate, compiler-verified code definitions.

## Technical Implementation

Standard Zed utilizes a background regex-based semantic indexer. This fork disables that subsystem and replaces it with `crates/agent/src/tools/context_tool.rs`. 

When the agent needs to understand a symbol, it forwards the request directly to the active server (`rust-analyzer`, `pyright`, etc.). This ensures the agent is reading the **actual resolved AST** of your project, not just a text-based guess.

## Usage Examples

### Example 1: Locating Type Definitions
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

### Example 2: Understanding Trait Implementations
<ZedChat>
  <template #user>
    Show me all implementations of the `Render` trait for components in `crates/ui`.
  </template>
  <template #assistant>
    Requesting LSP implementation map for `Render` trait...
  </template>
  <template #output>
    @context({ query: "Render" })
    >> Impl 1: crates/ui/src/button.rs
    >> Impl 2: crates/ui/src/modal.rs
  </template>
</ZedChat>

## Workflow Impact & Impact
- **Zero Hallucination**: The agent sees exactly what the compiler sees. It never guesses a file path or a symbol location.
- **Contextual Precision**: Understands the difference between `model.save()` and `result.save()` even if they share the same method name.
- **Zero-Config Performance**: Works instantly with your existing LSP setup (`rust-analyzer`, `gopls`, etc.) without needing a background index.
- **Massive Scale**: Can find a needle in a haystack across 1M+ lines of code by offloading the heavy lifting to the Language Server.
