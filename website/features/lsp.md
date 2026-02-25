# ⚡ LSP Symbol Search

The standard Zed AI agent relies extensively on regex-driven background indexing to traverse your codebase. While functional for small setups, indexing massive monolithic frameworks causes significant CPU spikes, laptop heat, and delayed search results.

This custom fork completely rips out the regex strategy.

## Native Language Server Protocol (LSP) Integration

Language Servers already hold an encyclopedic, purely typed understanding of your code hierarchy (e.g., `rust-analyzer` for Rust, `pyright` for Python, `gopls` for Go). 

This fork intercepts the agent's context requests and routes them directly to the native LSP layer.

### Benefits
- **Zero Background Indexing**: Eliminates the persistent background parsing threads. CPU stays entirely flat until you actively ask a question.
- **Perfect Accuracy**: The agent understands references, type traits, and implementations rather than just fuzzy text matching.
### Practical Context Example

Consider trying to refactor a deeply nested Rust interface. Standard background search might fail if the function is generated via a macro or shares a common name like `init`. 

With this fork's LSP integration, the agent behaves like your compiler:

> **You:** "What traits does `AgentModal` implement, and where are they?"
> 
> **Agent:** *(Queries the LSP using `context_tool`)* "The Language Server indicates `AgentModal` implements `Render` and `EventEmitter<AgentEvent>`. The `Render` implementation is found in `crates/agent_ui/src/modal.rs:45`, and the `EventEmitter` trait is defined natively by GPUI."

This guarantees the model isn't hallucinating references—it is strictly reading from the active compiler state.
