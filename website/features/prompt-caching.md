# Smart Prompt Caching

Zed Custom implements a sophisticated **Smart Caching** strategy designed to minimize both latency and API costs when using Anthropic's Claude models (including via Azure).

## How it Works

The editor automatically analyzes your conversation history and strategically places `cache_control` markers on parts of the prompt that are likely to be reused in future turns. This ensures that the bulk of your context (code, logs, and previous messages) stays in the model's high-speed memory.

### Caching Strategy

The "Smart Caching" logic follows a four-tiered approach:

1.  **System Prompt (Always Cached)**
    - The core system instructions and your persona are always marked for caching. Since these never change, they are almost 100% effective at reusing the cache.

2.  **Interval Checkpoints (Every 15 Blocks)**
    - As the conversation grows, Zed places a cache marker every 15 message blocks. This creates "anchors" in the timeline, ensuring that even in very long sessions, large chunks of the history remain cached.

3.  **Midpoint Anchoring**
    - For conversations exceeding 30 messages, the midpoint is explicitly cached. This provides a stable reference point for the model to look back on, preventing "cache evaporation" in extremely deep threads.

4.  **Immediate Prefix (Second-to-Last Message)**
    - The most recent context is the most valuable. Zed always caches the message immediately preceding your current input. This ensures that the "immediate past" is ready for instant reuse the moment you hit send.

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
