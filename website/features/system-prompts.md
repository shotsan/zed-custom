# 🤖 System Prompts & Persona

A great AI agent is defined by its core instructions. Upstream Zed provides a solid baseline, but to support our custom features (like long-term memory, headless browsing, and tool execution), we had to completely overhaul the system prompt architecture.

## The Prompt Template Engine

The core system prompts are managed in `crates/agent/src/templates.rs` and injected dynamically using Handlebars (`.hbs`) templates.

Our custom `system_prompt.hbs` is broken down into several distinct sections to ensure the LLM strictly adheres to its persona and properly leverages its environment.

### 1. The Persona & Tool Block
We instruct the LLM on exactly *who* it is and *how* it should act, followed by dynamically listing every custom tool the agent has available (e.g., `remember`, `recall`, `search`, `edit_file`). 

### 2. Epistemic State (Project Health)
The prompt injects real-time sensory context from Zed's LSP, instantly feeding the agent the active file path, compiler error counts, and warning metrics.

### 3. Modifiers (Python & C++ Best Practices)
Depending on the active files in your project, the engine injects explicit coding standards (e.g., forcing Pydantic for Python, or smart pointers for Modern C++).

### 4. The Active SQLite Memory Block
This is critical. Right before the user's constraints, the engine queries the SQLite memory database and injects all relevant project memories. It dynamically iterates over the 5 categories (`Architecture`, `Patterns`, `Issues`, `Procedures`, `Notes`) and formats them as headers directly in the prompt.

**Template Injection (`system_prompt.hbs:260`):**
```hbs
\{{#if (gt (len memories) 0)}}
## Project Memory
The following information has been remembered from previous sessions:

\{{#each memories}}
### \{{category}}
\{{content}}
\{{/each}}
\{{/if}}
```

### 5. Rule Injection (`.rules` files)
At the very bottom of the system prompt (maximizing the LLM's "recency" attention), the engine injects the contents of any `.rules` files found in your workspace root.

## Modifying the Persona

Because the templates are embedded via `rust_embed`, if you want to alter the fundamental persona of the agent, you must:
1. Edit the `crates/agent/src/templates/system_prompt.hbs` file.
2. Recompile the Zed binary (`cargo build --release`).
