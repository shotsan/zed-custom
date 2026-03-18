# 📉 Auto Context Compression

When you work on long-running tasks or massive refactoring sessions, your conversation logs and tool results can rapidly bloat the LLM's context window. Hitting a 200k token limit can cause the model to crash, drop messages, or cost an enormous amount of API credits on every turn.

To prevent this, Zed Custom implements a built-in **Auto Context Compression** mechanism directly into the thread logic.

## How it works

When the active thread's word count exceeds **70,000 words** (roughly ~100k tokens depending on the code structure), Zed silently initiates a background condensation process to ensure you never hit the 200k token ceiling.

Here is the exact step-by-step workflow of the compression feature:

1. **Context Preservation**: The agent retains the **last 15 messages** perfectly intact. This ensures that immediate debugging context, recent thoughts, and the current task state are entirely lossless.
2. **Archiving**: All messages preceding those last 15 are drained from the immediate message queue.
3. **Dense Summarization**: Zed seamlessly spins up a background LLM completion request to summarize the drained messages. The internal prompt specifically instructs the model to extract:
   - Files modified and the nature of the changes
   - Core architectural decisions made
   - Current blockers and known bugs
   - The user's ultimate goal
4. **System Prompt Injection**: The resulting summary is appended to your thread's `archive_summary` state and is injected directly into the overarching `# System Prompt` via handlebars (`\{\{\{archive_summary\}\}\}`) on all future requests.

## Why this matters

Instead of losing your place in a massive refactoring task or being forced to start a new chat, Zed condenses the historical timeline into a set of dense facts. The model never forgets *what* you were doing or *why*, but it successfully frees up nearly 100k tokens of space for new code generation.

This ensures you can keep coding indefinitely in the exact same thread without ever hitting the "Context Length Exceeded" error or suffering from extreme slow-downs.
