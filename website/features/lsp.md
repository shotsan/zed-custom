# ⚡ LSP Symbol Search

The standard Zed AI agent relies extensively on regex-driven background indexing to traverse your codebase. While functional for small setups, indexing massive monolithic frameworks causes significant CPU spikes, laptop heat, and delayed search results.

This custom fork completely rips out the regex strategy.

## Native Language Server Protocol (LSP) Integration

Language Servers already hold an encyclopedic, purely typed understanding of your code hierarchy (e.g., `rust-analyzer` for Rust, `pyright` for Python, `gopls` for Go). 

This fork intercepts the agent's context requests and routes them directly to the native LSP layer.

### Benefits
- **Zero Background Indexing**: Eliminates the persistent background parsing threads. CPU stays entirely flat until you actively ask a question.
- **Perfect Accuracy**: The agent understands references, type traits, and implementations rather than just fuzzy text matching.
- **Instant Synchronization**: As long as the LSP is attached, the AI agent has the absolutely latest representation of the code exactly as you type it.
