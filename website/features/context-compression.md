# 📉 Auto Context Compression

When you work on long-running tasks or massive refactoring sessions, your conversation logs and tool results can rapidly bloat the LLM's context window. This leads to **context forgetting** and high **model token** consumption on every turn.

To prevent this, the IDE implements a built-in **Auto Context Compression** mechanism directly into the thread logic.

---

## How it works

When the thread exceeds **70,000 words** (including code mentions), the IDE automatically condenses the context in the background. This ensures that the model always has room for new instructions without losing the thread's technical history.

### The Workflow:
1.  **Preserve Context**: We keep only the **last 2 messages** to maximize space recovery while maintaining immediate continuity.
2.  **Consolidated Summary**: The old summary is merged with new messages to create a **single, dense technical snapshot** of the task.
3.  **Recursive State**: The new summary replaces the old, ensuring you never hit context length limits even in massive refactoring sessions.

---

## Why this matters

Instead of losing your place in a massive refactoring task or being forced to start a new chat, the IDE condenses the historical timeline into a set of dense facts. The model never forgets *what* you were doing or *why*, but it successfully clears the context window for your next turn.

This ensures you can keep coding indefinitely in the exact same thread without suffering from slow-downs or context-related inaccuracies.
