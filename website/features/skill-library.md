# 🧠 Skill Library: The AI's Expertise Engine

The **Skill Library** is the foundational system that transforms a generic Large Language Model into a **specialized software engineer** tailored to your specific stack, architecture, and team conventions.

---

## 🚀 Instant Discovery

Zed Custom makes your expertise discoverable and reachable in seconds.

-   **Command Palette**: Search for "Skill Library" or hit `cmd-shift-s`.
-   **Slash Commands**: Every skill in your library is automatically available as a top-level command. Type `/rust` or `/tailwind` directly in the chat.
-   **Contextual UI**: Manage your skills in a dedicated tab with a full IDE-grade editor.

---

## 📂 Hybrid Expertise System

Skills are managed through two parallel systems, giving you the best of both world: **Shared Team Standards** and **Personal Library expertise**.

### 1. Global Skill Library (Personal)
Stored in a local high-performance database, these skills follow you across all your projects.
- **Pinned Skills**: Toggle the **Paperclip icon** to include a skill in the permanent system prompt of every conversation.
- **Expert Templates**: Comes pre-loaded with experts for Rust Security, Tailwind CSS, and Commit Message generation.

### 2. Project-Specific Skills (Team)
Zed Custom natively supports "File-based" skills that you can check into Git. These are automatically detected and prioritized:
- **`.rules` / `.cursorrules`**: Global project instructions.
- **`AGENT.md`**: Best for high-level architectural constraints.
- **`.python_rules` / `.cpp_rules`**: Language-specific expertise injected only when relevant.

---

## 🛠️ Managing Your Skills

<div class="features-grid">
  <div class="feature-card">
    <h3>📝 Rich Editor</h3>
    <p>Edit your prompts with full syntax highlighting, auto-indentation, and Markdown support.</p>
  </div>
  <div class="feature-card">
    <h3>🪄 Inline Refinement</h3>
    <p>Highlight any part of your skill and use <code>cmd-i</code> to have the AI help you rewrite or clear up your instructions.</p>
  </div>
  <div class="feature-card">
    <h3>🔗 Pinned Persistence</h3>
    <p>Pinned skills benefit from "Recency Bias" injection, ensuring your most important constraints are never forgotten by the model.</p>
  </div>
</div>

---

## ⚡ Real-World Example

If you have a **"Performance Expert"** skill pinned with these instructions:
> "Always optimize for zero-allocation paths. Prefer `SmallVec` over `Vec` for small lists."

The agent will automatically align its code suggestions without you ever mentioning performance in the chat:

<div class="zed-chat-mockup">
  <div class="user-msg">"Write a function to collect 4 string IDs."</div>
  <div class="assistant-msg">
    <small>Consulting <b>Performance Expert</b> skill...</small><br/>
    "Using <code>SmallVec&lt;[String; 4]&gt;</code> to ensure zero heap allocations for this fixed-size list."
  </div>
</div>

---

## 💎 Pro-Tips for High-Quality Docs

-   **Precedence**: Project files always overwrite Library skills of the same name.
-   **Scoping**: Use directory-level `.rules` files to give sub-modules their own specialized logic.
-   **Slugification**: A skill named "Rust Security Auditor" will always be available as `/rust-security-auditor`.

---

> [!TIP]
> **Getting Started**: Create a new skill called "My Preferences." Add instructions like "I prefer explicit error handling over <code>.unwrap()</code>" and pin it with the paperclip. Your AI will immediate feel like it has been working with you for years.
