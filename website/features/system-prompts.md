# 🎭 System Prompts & Persona

The System Prompt is the agent's "subconscious"—a set of foundational instructions that define its identity, its available tools, and its behavioral boundaries.

## The Handlebars Engine

Instead of a static text file, this fork uses the **Handlebars** template engine to dynamically construct the prompt based on your real-time environment. 

The template is located in `crates/agent/src/templates/system_prompt.hbs` and is embedded into the binary using `rust-embed` for maximum performance.

## Prompt Prioritization (Top to Bottom)

The prompt is structured into five distinct sections, prioritized by their distance from the user's latest query:

1.  **Identity & Persona**: Defines the agent as a "10x proactive engineer" who uses `<thought>` blocks.
2.  **Epistemic State**: Dynamic sensors providing real-time data on active files and LSP error counts.
3.  **Language Modifiers**: Targeted best practices injected based on the file extension (e.g., C++17 pointers).
4.  **Project Memory**: Contextually relevant facts retrieved from the [SQLite database](/features/memory).
5.  **Custom Rules**: Your project-scoped `.rules` files, placed last to maximize the model's attention.

## Technical Detail: Memory Injection

The memory block is injected using a conditional Handlebars loop that iterates through your project's historical facts:

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

## Modifying the Persona

Because the prompt is embedded in the binary, fundamental changes to the agent's base persona require editing the `.hbs` template and re-running `cargo build --release`. 

For transient project-specific behaviors, we recommend using [Custom Rules](/features/rules) instead of binary modifications.
