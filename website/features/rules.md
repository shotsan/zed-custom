# 📝 Custom Rule Insertion

A major pain point with standard AI coding assistants is their inability to consistently follow a team's specific coding guidelines or architectural constraints. 

To solve this, our Zed agent natively supports **Custom Rule Insertion** via `.rules` files.

# 📝 Custom Rules (.rules)

Tailor the AI agent's behavior by placing `.rules`, `.cpp_rules`, or `.python_rules` files in your project root.

## Rule Injection

These files are injected at the very bottom of the [System Prompt](/features/system-prompts), giving them the highest "recency bias" for the LLM.

### Example: `.python_rules`
```md
- Always use Pydantic v2 for data models.
- Type hints are mandatory for all public APIs.
```

## Strategy: Rules vs Memory

| Strategy | Usage | Injection Point |
| :--- | :--- | :--- |
| **`.rules` Files** | Permanent team standards | Bottom of Prompt (Highest Priority) |
| **SQLite Memory** | Evolving codebase facts | Middleware (Contextual) |

<ZedChat>
  <template #user>
    Write a user model.
  </template>
  <template #assistant>
    Consulting `.python_rules`...
    Implementing with Pydantic v2 and mandatory type hints.
  </template>
</ZedChat>
