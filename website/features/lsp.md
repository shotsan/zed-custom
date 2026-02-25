# ⚡️ LSP Symbol Context

To minimize CPU overhead and maximize accuracy, we replaced Zed's legacy background Regex indexing with a native **Language Server Protocol (LSP)** integration.

## Accuracy Over Hallucination

Standard AI editors often guess file paths or literal string matches, leading to hallucinations in complex codebases. By leveraging your active language server (`rust-analyzer`, `gopls`, `pyright`, etc.), our agent obtains a perfectly typed and fully resolved map of your project.

### Key Logic: `context_tool.rs`
Implemented in `crates/agent/src/tools/context_tool.rs`, the system intercepts agent context requests and routes them directly to the active LSP instance. If the compiler knows where the definition is, the agent knows where the definition is.

## Visual Walkthrough

### Querying Code Relationships

<ZedChat>
  <template #user>
    Where is the `UserSession` struct defined and how is it used in the authentication flow?
  </template>
  <template #assistant>
    Requesting LSP symbol map for `UserSession`...
  </template>
  <template #output>
    @context({ query: "UserSession" })
    >> Definition: crates/core/src/auth.rs:L45
    >> Reference 1: crates/api/v1/login.rs:L89
    >> Reference 2: main.rs:L127
  </template>
</ZedChat>

## Why it matters
- **Zero CPU Waste**: Eliminates the persistent background parsing threads that drain battery life.
- **Recursive Accuracy**: Understands trait implementations, macros, and complex generics that Regex-based indexers miss.
- **Instant Sync**: As soon as you save a file, the agent's "knowledge" is updated via the LSP.

This guarantees the model isn't hallucinating references—it is strictly reading from the active compiler state.
