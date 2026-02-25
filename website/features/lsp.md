# ⚡️ LSP Symbol Context

To minimize CPU overhead and maximize accuracy, we replaced Zed's legacy background Regex indexing with a native **Language Server Protocol (LSP)** integration.

## Accuracy Over Hallucination

Standard AI editors often guess file paths or literal string matches, leading to hallucinations in complex codebases. By leveraging your active language server (`rust-analyzer`, `gopls`, `pyright`, etc.), our agent obtains a perfectly typed and fully resolved map of your project.

### Key Logic: `context_tool.rs`
Implemented in `crates/agent/src/tools/context_tool.rs`, the system intercepts agent context requests and routes them directly to the active LSP instance. If the compiler knows where the definition is, the agent knows where the definition is.

## Visual Walkthrough

### Example 1: Finding Interface Implementations
<ZedChat>
  <template #user>
    Which types in this project implement the `LanguageModel` trait?
  </template>
  <template #assistant>
    Requesting LSP implementations map for trait `LanguageModel`...
  </template>
  <template #output>
    @context({ query: "LanguageModel" })
    >> Impl 1: crates/language_model/src/db_model.rs
    >> Impl 2: crates/anthropic/src/anthropic.rs
    >> Impl 3: crates/google/src/gemini.rs
  </template>
</ZedChat>

### Example 2: Resolving Macro-Generated Code
<ZedChat>
  <template #user>
    Show me where the `gpui::actions!` macro is defined and how it processes the `Quit` action.
  </template>
  <template #assistant>
    Querying LSP for macro definition and expansion site...
  </template>
  <template #output>
    @context({ query: "actions!" })
    >> Definition: crates/gpui/src/action.rs:L12
    >> Expansion: crates/zed/src/main.rs:L45 (Quit Action generated here)
  </template>
</ZedChat>

## Workflow Impact
- **Zero CPU Waste**: Eliminates the persistent background parsing threads that drain battery life.
- **Recursive Accuracy**: Understands trait implementations, macros, and complex generics that Regex-based indexers miss.
- **Instant Sync**: As soon as you save a file, the agent's knowledge is updated via the LSP.
- **Deep Refactoring Support**: The agent can accurately find every single reference of a symbol across the entire project before proposing a change.

This guarantees the model isn't hallucinating references—it is strictly reading from the active compiler state.
