# ⚡ Tool: `context`

The `context` tool is the powerhouse underlying the Custom Zed LSP integration. It explicitly maps the agent's understanding of the codebase down to the Language Server Protocol.

## Technical Details

**Internal File:** `crates/agent/src/tools/context_tool.rs`

Standard Zed uses an expensive RegEx background indexer. This fork disables that subsystem and replaces it with `context_tool.rs`. When the LLM wants to find where a function is defined or wants to map the usages of a trait, it passes the symbol to the `context` tool. 

The `context` tool forwards this request natively to `rust-analyzer`, `pyright`, or `gopls`. The Language Server, which already holds a perfectly typed and fully resolved AST of your project in memory, returns the precise definitions instantly.

## Use Cases

Instead of guessing file paths or doing literal string matches, the AI operates like a compiler.

```md
User: "Where is the `spawn` method defined for the Executor?"
Agent: [Invokes context(symbol="Executor::spawn")]
Agent: "The LSP found it in `crates/gpui/src/executor.rs` on line 112..."
```
