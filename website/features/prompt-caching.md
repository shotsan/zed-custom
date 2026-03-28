# Smart Prompt Caching

Zed Custom implements a sophisticated **Smart Caching** strategy designed to minimize both latency and API costs when using Anthropic's Claude models (including via Azure).

## How it Works

The editor automatically analyzes your conversation history and strategically places `cache_control` markers on parts of the prompt that are likely to be reused in future turns. This ensures that the bulk of your context (code, logs, and previous messages) stays in the model's high-speed memory.

### Caching Strategy

The "Smart Caching" logic follows a four-tiered approach:

1.  **System Prompt (Always Cached)**
    - The core system instructions and your persona are always marked for caching.

2.  **Last History Item (Turn-over-Turn)**
    - Zed explicitly caches the last message in your history. This ensures that as your conversation grows, the "immediate past" remains hot for the next turn.

3.  **Immediate Prefix (Active Tool Loops)**
    - To speed up parallel tool execution and recursive research turns, the current message prefix is cached. This ensures tool results can be processed without re-parsing the entire prompt context.

## The Visual Indicators

When you are in a session with more than **1024 tokens** (Anthropic's minimum caching threshold), you will see real-time feedback in the turn stats:

- 🟢 **[X]k cached**: Tokens that were reused from previous turns. You aren't being charged full price for these!
- 🔵 **+[X]k saved**: New tokens from the current turn that have been written into the cache for future use.

## Technical Implementation

The logic is implemented in the `agent` crate:

```rust
// crates/agent/src/thread.rs

const CACHE_INTERVAL: usize = 15;
// ... logic for interval, midpoint, and prefix caching ...
```

By ensuring that the prefix of the prompt remains largely static turn-over-turn, we maximize the **Cache Hit Rate**, results in near-instant responses even for 100k+ token threads.
