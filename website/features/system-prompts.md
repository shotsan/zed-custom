# 🎭 System Prompts & Persona

The custom `system_prompt.hbs` template defines the agent's core identity and behavioral constraints.

## Prompt Structure

The prompt is constructed in five prioritized blocks:

1.  **Persona**: Defines the agent as a "10x proactive pair programmer."
2.  **Epistemic State**: Real-time diagnostic counts (errors/warnings) from the LSP.
3.  **Modifiers**: Language-specific best practices (e.g., Modern C++ standards).
4.  **SQLite Memory**: Injected facts from your `@remember` history.
5.  **Custom Rules**: The contents of your project's `.rules` files.

## Memory Category Injection

Dynamic memories are injected using the following Handlebars logic:

```hbs
\{{#each memories}}
### \{{category}}
\{{content}}
\{{/each}}
```

This ensures the agent receives context categorized into **Architecture**, **Patterns**, **Issues**, **Procedures**, or **Notes** before processing your request.
