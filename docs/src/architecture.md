# Zed Architecture

Zed is built from the ground up in Rust to be a high-performance, multiplayer code editor. Its architecture is designed to maximize responsiveness, efficiently utilize modern multi-core CPUs, and natively support real-time collaboration.

This document provides a deep dive into the core architectural choices and components that make Zed work.

## 1. GPUI: The UI Framework

At the heart of Zed is **GPUI** (GPU UI), a custom-built, retained-immediate hybrid UI framework. Because standard Rust UI frameworks did not meet the performance and architectural needs for a real-time multiplayer editor, Zed Industries built GPUI.

### Core Concepts of GPUI

- **Reactivity and Entities**: All stateful UI components in Zed are modeled as "Entities". An `Entity<T>` is a handle to a piece of data of type `T`. 
- **Contexts (`cx`)**: State management in Rust often battles the borrow checker. GPUI solves this by passing around a context (`&mut App`, `&mut WindowContext`, or `&mut Context<T>`). Instead of components holding direct references to each other, they hold `Entity` handles and use the context to read or update the underlying data (e.g., `entity.read(cx)` or `entity.update(cx, |data, cx| ...)`).
- **GPU Rasterization**: Drawing is highly optimized. GPUI converts the UI into a tree of display items, which are batched and directly rasterized onto the GPU using Metal (macOS), Vulkan, or DirectX, ensuring consistent 120fps+ rendering.
- **Single Foreground Thread**: All UI rendering and entity mutations occur on a single main thread to prevent race conditions. CPU-heavy work (like parsing or network requests) is offloaded using `cx.background_spawn()`, which returns a `Task` that the main thread can await.

## 2. Project and Worktree

Zed models the user's workspace using the `Project` and `Worktree` crates.

- **Worktree**: A `Worktree` represents a directory on disk (or a remote directory via SSH/LiveKit). It watches the file system for changes (`fsevents` on macOS) and maintains an in-memory snapshot of the directory tree.
- **Project**: The `Project` is the highest-level state container for a workspace. It holds multiple `Worktree`s, manages the active Language Server Protocol (LSP) clients, and maintains the shared semantic understanding of the code.

Whenever you open a folder in Zed, a `Project` is created, which spawns the necessary `Worktree`s and bootstraps the language servers for the detected files.

## 3. Text Buffers and CRDTs

Zed was designed from day one to be multiplayer. To achieve instant, conflict-free collaboration, Zed uses Conflict-free Replicated Data Types (CRDTs) to manage text and state.

- **The `sum_tree`**: The foundational data structure in Zed is the `SumTree` (a counted B-tree). It is used for almost everything: the text buffer itself, the list of compiler diagnostics, and the Git blame annotations. It allows for extremely fast $O(\log n)$ insertions, deletions, and lookups.
- **Buffers and Operations**: When you type in Zed, you are not just mutating a string. You are generating an `Operation`. These operations are applied locally to your `Buffer` immediately, giving you zero-latency typing. 
- **Multiplayer Sync**: In the background, these `Operation`s are serialized and sent over a WebSocket connection to the Zed `collab` server. The server broadcasts them to other users in the session. Because they are CRDTs, operations can arrive out of order, but every client will mathematically converge on the exact same text state without needing a central locking mechanism.

## 4. Multi-buffers

One of Zed's unique architectural choices is the **Multi-buffer**. 

A `MultiBuffer` is a virtual text buffer composed of slices from many different physical files. For example, when you use "Find All References", Zed creates a single `MultiBuffer` containing the excerpts of every file where that reference exists. 

Because the `MultiBuffer` acts like a standard `Buffer`, the editor can render it seamlessly. If you type in a MultiBuffer, the edits are automatically mapped back to the underlying physical files and their respective CRDT `Operation` streams.

## 5. Parsing & Syntax (Tree-sitter)

Instead of relying on slow, inaccurate Regex-based syntax highlighting (like TextMate grammars), Zed uses **Tree-sitter** for everything.

- **Incremental Parsing**: As you type, Tree-sitter incrementally reparses just the affected AST nodes on a background thread.
- **Semantic Highlighting**: The AST provides a deep semantic understanding of the code. The editor queries the AST to accurately color variables, functions, and keywords, ensuring the syntax highlighting never breaks, even if the code contains syntax errors.
- **Structural Tools**: Features like "Extend Selection" or "Go to Symbol" operate directly on the Tree-sitter AST layer rather than relying purely on text offsets.

## 6. Extension Architecture

Zed's extension architecture uses **WebAssembly (Wasm)**.

When you install a Zed extension, you are downloading a pre-compiled `.wasm` binary. Zed executes these extensions inside a secure, sandboxed Wasm runtime (`wasmtime`). 
- **Why Wasm?** It provides a high-performance execution environment while ensuring extensions cannot crash the editor, access arbitrary files on your system without permission, or steal environment variables.
- **Extension Capabilities**: Extensions can register new Tree-sitter grammars, inject language servers, provide custom themes, or introduce new slash commands for the AI Assistant.

## Conclusion

Zed achieves its speed by tightly coupling a custom GPU-accelerated UI framework with highly optimized data structures (Counted B-Trees & CRDTs), while offloading heavy lifting (Tree-sitter parsing & Language Servers) to background threads. Its multiplayer capabilities are not bolted on, but baked into the foundational text buffer architecture.
