# 🔬 Internals: Tree-sitter, AST, LSP & Code Indexing

This page explains the four technologies that work together to give the agent its understanding of your code. Each one operates at a different layer and solves a different problem.

---

## Abstract Syntax Tree (AST)

An **Abstract Syntax Tree** is the data structure that results from parsing source code. Instead of treating a file as a flat string of characters, a parser reads the code and builds a tree where each node represents a meaningful unit of the language — a function definition, a block, an expression, an identifier.

```
// Source text
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// What the AST looks like (simplified)
function_item
  name: identifier "add"
  parameters: parameters
    parameter: identifier "a" type: primitive_type "i32"
    parameter: identifier "b" type: primitive_type "i32"
  return_type: primitive_type "i32"
  body: block
    binary_expression
      left: identifier "a"
      right: identifier "b"
```

The AST **removes all syntactic noise** (whitespace, comments, parentheses used purely for grouping) and retains only the semantic structure of the code. All the other tools described below — Tree-sitter, and LSP — are built on top of some form of this structure.

---

## Tree-sitter

**Tree-sitter** is a fast, incremental parser that builds an AST from source text. It is embedded directly into Zed as a Rust library (`crates/language/src/language.rs` exports `tree_sitter::Node`, `Tree`, `Parser`, etc.).

### What it does

Tree-sitter parses a file into an AST using a **grammar** compiled for each language (e.g., `tree_sitter_rust::LANGUAGE`, `tree_sitter_python::LANGUAGE`). It is designed for editors: it is incremental, meaning it only re-parses the parts of a file that changed, not the whole document, making it fast enough to run on every keystroke.

Tree-sitter does **not** know anything about your project. It parses one file at a time, in isolation, with no knowledge of imports, types, or external definitions. It understands syntax — not semantics.

### Where this repo uses it

Every open buffer in Zed maintains a `TreeSitterData` struct (see `crates/language/src/buffer.rs`). This gives each buffer a live, always-current syntax tree.

The specific place this matters for the agent is **file outline generation**, implemented in `crates/agent/src/outline.rs`:

```rust
// From outline.rs
pub const AUTO_OUTLINE_SIZE: usize = 16384; // 16 KB

// When a file is larger than 16 KB, the agent reads the tree-sitter
// outline instead of the full file content.
let outline_items = buffer.read_with(cx, |buffer, _| {
    let snapshot = buffer.snapshot();
    snapshot.outline(None).items  // <-- this calls into tree-sitter
        .into_iter()
        .map(|item| item.to_point(&snapshot))
        .collect::<Vec<_>>()
});
```

When the agent opens a file over 16 KB, it asks the buffer for its **outline** — a compact, tree-sitter-derived list of top-level symbols (functions, structs, impls, classes, etc.) with their line ranges. This is sent to the model instead of the full file content, keeping context usage bounded on large files.

If a file has no parseable tree-sitter outline (e.g., a plain text file or an unsupported language), the agent falls back to the first 1 KB of raw content.

### What Tree-sitter cannot do

- It cannot resolve a type reference across files.
- It cannot tell you where a function is defined if it's imported from another module.
- It cannot understand generics, trait bounds, or type inference.

For those capabilities, you need a Language Server.

---

## Language Server Protocol (LSP)

**LSP** is a standard JSON-RPC protocol that defines how an editor communicates with a language-specific analysis tool (a "language server"). The language server runs as a separate process alongside the editor. It has full knowledge of your project: it understands imports, resolves types, knows where every symbol is defined, and can answer queries like "find all references to this function."

Common language servers this repo works with:

| Language | Server |
|---|---|
| Rust | `rust-analyzer` |
| Python | `pyright` |
| Go | `gopls` |
| TypeScript/JavaScript | `typescript-language-server` |
| C/C++ | `clangd` |

### How the protocol works

When the agent needs to look up a symbol, it sends a request using Zed's existing LSP client infrastructure (`crates/project`). Zed already maintains persistent connections to all active language servers for the open project — the agent simply reuses those connections.

The LSP requests this fork uses for agent tools are:

| LSP Request | Tool | Description |
|---|---|---|
| `workspace/symbol` | `query_context` | Fuzzy symbol search across the whole project |
| `textDocument/definition` | `lsp_get_definition` | Jump-to-definition at a position |
| `textDocument/references` | `lsp_find_references` | Find all call sites/uses of a symbol |
| `textDocument/implementation` | `lsp_get_implementations` | Find all types implementing a trait/interface |

### `query_context` — workspace symbol search

The primary context tool calls `project.symbols(&query, cx)`, which sends a `workspace/symbol` LSP request to all active language servers with your query string. The server does a fuzzy match against its own symbol index (which it maintains from compiling/analyzing the full project) and returns matches with their file path, kind, and line range.

```rust
// From context_tool.rs
let task: Task<Result<Vec<Symbol>>> = cx.update(|cx| {
    project.update(cx, |project, cx| {
        project.symbols(&query, cx)  // LSP workspace/symbol request
    })
});
let symbols = task.await?;
```

Results are capped at 20 symbols to avoid overflowing the model's context window.

### `lsp_get_definition`, `lsp_find_references`, `lsp_get_implementations`

These three tools in `crates/agent/src/tools/lsp_tools.rs` take a **file path + line + column** as input, open that buffer if it isn't open yet, resolve the position to a `PointUtf16`, and forward a targeted LSP request (`definition`, `references`, or `implementation`) to the relevant language server. Each returns a list of file paths with line and column positions.

All three are cancellable — if you click "Stop" in the agent panel mid-tool, the LSP request is abandoned via `event_stream.cancelled_by_user()`.

---

## Code Indexing

This repo contains two separate symbol indexing mechanisms. Understanding which one is actually active matters.

### 1. The `SemanticIndex` (regex-based, currently dormant)

`crates/agent/src/semantic_search.rs` defines a `SemanticIndex` struct that stores a `HashMap<String, Vec<SymbolLocation>>`. Its `build()` method walks the project's worktrees, finds supported source files (`.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, `.cs`, `.cpp`, etc.), reads each one from disk, and extracts symbol names using **regex patterns** — not tree-sitter, not LSP.

```rust
// Python example from semantic_search.rs
let function_pattern = Regex::new(r"(?m)^\s*def\s+(\w+)\s*\(")?;
let class_pattern    = Regex::new(r"(?m)^\s*class\s+(\w+)")?;
```

Each language has its own set of regex patterns. The result is a flat in-memory map from symbol name → list of locations that can be queried for prefix and substring matches.

**However, this index is deliberately not running.** In `agent.rs`:

```rust
// agent.rs — NativeAgent::new()
let semantic_index = Arc::new(parking_lot::RwLock::new(SemanticIndex::new()));
let maintain_semantic_index = Task::ready(Ok(()));  // <-- no-op, nothing is scheduled
```

The `SemanticIndex` struct is instantiated (and its `symbol_count()` is exposed in the status panel), but the background task that would call `SemanticIndex::build()` and keep it updated is replaced with a task that immediately resolves to `Ok(())`. **The index is always empty in normal operation.**

This was a deliberate trade-off: the regex indexer was expensive to maintain (CPU, disk I/O on large projects) and less accurate than LSP. It is kept in the codebase but disabled in favour of the LSP path.

### 2. Tree-sitter File Outlines (active, per-file)

As described in the Tree-sitter section, every open buffer maintains its own tree-sitter parse tree. This is **not** a project-wide index — it only covers files that are currently open as buffers. Its output is used exclusively by the `read_file` tool flow in `outline.rs` to avoid flooding the model's context with a full large file.

### 3. LSP Workspace Symbol Index (active, maintained by language server)

The language server itself maintains a full project-wide symbol index as part of its normal operation. When `rust-analyzer` starts, it indexes your entire Rust workspace. When `gopls` starts, it indexes all Go packages. This index is updated incrementally as you edit files.

This is the index the agent actually uses via `project.symbols()`. It is never built or managed by Zed or this fork — it lives entirely inside the language server process.

---

## Summary: Which Technology Answers Which Question

| Question | Technology | Active? |
|---|---|---|
| Syntax highlight this token | Tree-sitter | ✅ Always |
| What symbols are in this large file? | Tree-sitter outline | ✅ When file > 16 KB |
| Where is `MyStruct` defined? | LSP `workspace/symbol` | ✅ When a server is running |
| Where is the definition at line 42, col 8? | LSP `textDocument/definition` | ✅ When a server is running |
| Who calls this function? | LSP `textDocument/references` | ✅ When a server is running |
| What types implement this trait? | LSP `textDocument/implementation` | ✅ When a server is running |
| In-process regex symbol search | `SemanticIndex` | ⚫ Disabled (always empty) |
