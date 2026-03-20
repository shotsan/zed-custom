# 📉 Auto Context Compression

When you work on long-running tasks or massive refactoring sessions, your conversation logs and tool results can rapidly bloat the LLM's context window. Hitting a 200k token limit can cause the model to crash, drop messages, or cost an enormous amount of API credits on every turn.

To prevent this, **Zed Custom** implements a built-in **Auto Context Compression** mechanism directly into the thread logic.

## How it works

When the thread exceeds **70,000 words** (including code mentions), **Zed Custom** automatically condenses the context in the background.

### The Workflow:
1. **Preserve Context**: We keep only the **last 2 messages** to maximize space recovery while maintaining immediate continuity.
2. **Consolidated Summary**: The old summary is merged with new messages to create a **single, dense technical snapshot** of the task.
3. **Recursive State**: The new summary replaces the old, ensuring you never hit context length limits even in massive refactoring sessions.

## Why this matters

Instead of losing your place in a massive refactoring task or being forced to start a new chat, **Zed Custom** condenses the historical timeline into a set of dense facts. The model never forgets *what* you were doing or *why*, but it successfully clears the context window for your next turn.

This ensures you can keep coding indefinitely in the exact same thread without ever hitting "Context Length Exceeded" errors or suffering from extreme slow-downs.
