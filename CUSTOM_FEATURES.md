# Zed Custom - Enhanced AI Agent Features

This is a custom fork of [Zed](https://github.com/zed-industries/zed) with enhanced AI agent capabilities focused on improving code understanding and long-term memory.

## 🚀 Custom Features

### 1. **LSP-Based Symbol Search** 
**Status:** ✅ Implemented

Replaced the original regex-based semantic indexing with native Language Server Protocol (LSP) integration for accurate, language-aware symbol navigation.

**Benefits:**
- ⚡ **Faster**: No CPU-intensive background indexing
- 🎯 **More Accurate**: Leverages language servers (rust-analyzer, pyright, gopls, etc.)
- 🔧 **Type-Aware**: Understands code structure, not just text patterns
- 🔄 **Always Up-to-Date**: Automatically maintained by LSP as you edit

**Implementation:**
- [`crates/agent/src/tools/context_tool.rs`](crates/agent/src/tools/context_tool.rs) - New LSP-based context tool
- [`crates/agent/src/agent.rs`](crates/agent/src/agent.rs) - Disabled legacy regex indexing
- [`crates/agent/src/thread.rs`](crates/agent/src/thread.rs) - Updated tool initialization

**Commit:** [`91b74d68c6`](https://github.com/shotsan/zed-custom/commit/91b74d68c6)

---

### 2. **Long-Term Memory System** 
**Status:** ✅ Implemented

AI agent can now remember and recall information across sessions using a persistent SQLite database.

**Features:**
- 💾 **Persistent Storage**: Memories survive across editor restarts
- 🏷️ **Categorized**: Architecture, Patterns, Issues, Procedures, Notes
- 🔍 **Searchable**: Query memories by content or category
- 📊 **Project-Scoped**: Memories are tied to specific projects

**Agent Tools:**
- `remember` - Store important information for future sessions
- `recall` - Retrieve previously stored memories

**Implementation:**
- [`crates/agent/src/memory_store.rs`](crates/agent/src/memory_store.rs) - SQLite-based memory database
- [`crates/agent/src/tools/memory_tools.rs`](crates/agent/src/tools/memory_tools.rs) - Remember/Recall agent tools

**Example Usage:**
```
User: "Remember that this project uses a microservices architecture with gRPC for inter-service communication"
Agent: Uses the 'remember' tool to store this in the Architecture category

[Later session]
User: "What architecture does this project use?"
Agent: Uses the 'recall' tool to retrieve the stored memory
```

---

### 3. **Enhanced UI Components**
**Status:** 🚧 In Development

Custom UI modals for managing agent memory and teaching custom rules.

**Components:**
- [`crates/agent_ui/src/acp/memory_manager_modal.rs`](crates/agent_ui/src/acp/memory_manager_modal.rs) - Memory management interface
- [`crates/agent_ui/src/acp/teach_rule_modal.rs`](crates/agent_ui/src/acp/teach_rule_modal.rs) - Rule teaching interface

---

## 🔧 Setup & Usage

### Building from Source

```bash
# Clone this repository
git clone https://github.com/shotsan/zed-custom.git
cd zed-custom

# Build Zed
cargo build --release

# Run
cargo run
```

### Syncing with Upstream Zed

This repository maintains `zed-industries/zed` as an upstream remote:

```bash
# Fetch latest from official Zed
git fetch upstream

# Merge upstream changes
git merge upstream/main

# Push to your fork
git push origin main
```

---

## 📝 Configuration Files

Custom project-specific rules can be defined in:
- `.cpp_rules` - C++ specific coding guidelines
- `.python_rules` - Python specific coding guidelines

---

## 🤝 Contributing

This is a personal fork with experimental features. If you find these features useful:
1. Consider contributing improvements via pull requests
2. Report issues specific to custom features
3. For core Zed issues, please report to [zed-industries/zed](https://github.com/zed-industries/zed)

---

## 📜 License

Same as [Zed](https://github.com/zed-industries/zed) - see their LICENSE files.

---

## 🔗 Links

- **Upstream Zed:** https://github.com/zed-industries/zed
- **This Fork:** https://github.com/shotsan/zed-custom

---

## 📊 Feature Comparison

| Feature | Upstream Zed | This Fork |
|---------|-------------|-----------|
| Symbol Search | Regex-based | ✅ LSP-based |
| Agent Memory | ❌ Session-only | ✅ Persistent SQLite |
| Background Indexing | Heavy CPU usage | ✅ Disabled (uses LSP) |
| Memory Categories | N/A | ✅ 5 categories |
| Cross-Session Context | ❌ | ✅ Yes |

---

**Last Updated:** 2026-02-09
