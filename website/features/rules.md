# 📝 Custom Rule Insertion

A major pain point with standard AI coding assistants is their inability to consistently follow a team's specific coding guidelines or architectural constraints. 

To solve this, our Zed agent natively supports **Custom Rule Insertion** via `.rules` files.

# 📝 Custom Rules (.rules)

Custom rules allow you to enforce project-specific coding standards, architectural constraints, and team preferences directly on the AI agent.

## Rule Detection & Injection

The agent's context engine (implemented in `crates/agent/src/templates.rs`) automatically scans your workspace root for specific rule files:
- **`.rules`**: Global rules applied to every turn.
- **`.cpp_rules`**: Specific logic injected only when a C++ file is in context.
- **`.python_rules`**: Specific logic injected only when a Python file is in context.

These files are injected at the **very bottom** of the system prompt. Because they appear last, they benefit from the "recency bias" of Large Language Models, ensuring your specific instructions take precedence over the agent's general training.

## Visual Walkthrough

### Enforcing Team Standards

If you have a `.python_rules` file containing:
```md
- Always use Pydantic v2 for data models.
- Type hints are mandatory for all public APIs.
```

The agent will automatically align its output without you needing to mention these rules in the chat:

<ZedChat>
  <template #user>
    Write a simple User model with an email field.
  </template>
  <template #assistant>
    Consulting `.python_rules` for project standards...
    Implementing with Pydantic v2 and mandatory type hints.
  </template>
</ZedChat>

## Strategy: Rules vs Memory

| **`.rules` Files** | Permanent team standards (Static) | Committed to Git |
| **SQLite Memory** | Evolving codebase facts (Dynamic) | Local Project DB |

## Workflow Impact
- **Team Standardization**: Commit a `.rules` file to your project root to ensure every developer's AI assistant follows the same linting, naming, and architectural standards.
- **Zero Hallucination**: Proactively tell the agent about obscure project "gotchas" before it suggests a fix that breaks your specific environment.
- **Automated Persona**: Define the agent's tone and expertise level on a per-directory basis (e.g., "Be a grumpy Senior C++ dev in `crates/engine`").
- **Language Specificity**: Add `.rules` to specific subdirectories to inject language-specific constraints only where they are needed.
