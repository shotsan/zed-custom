# 📝 System Prompts & Persona

System Prompts are the top-level instructions that define the Assistant's core behavior, tone, and technical constraints. This IDE grants you absolute control to modify or completely overwrite these instructions.

---

## 🛠️ The Prompt Hierarchy

When the Assistant generates a response, it synthesizes three distinct layers of instructions in order:

1.  **The Base System Prompt**: The fundamental logic that instructs the model on how to use tools, how to format responses, and its identity as an AI coding partner.
2.  **Global Instructions**: Your personalized technical rules (e.g., "Always use TypeScript," "Prefer functional components").
3.  **Project-Scoped Rules (`.rules` / `.cursorrules`)**: Specific logic that only applies to your current repository or directory.

---

## 🧠 Why Overwrite?

While the default logic is optimized for general engineering, advanced users can customize the Assistant's performance:

-   **Precision Control**: If the model is making recurring formatting errors or being overly chatty, you can modify the internal instructions directly.
-   **Specialized Models**: Some models (like `o1-preview`) respond better to extremely brief instructions. You can create a "Minimalist" profile specifically for these models.
-   **Security Constraints**: Explicitly forbid the use of certain libraries or external APIs in the Assistant's "foundational" logic.

---

## 🏗️ Technical Override Mechanism

You can override these prompts in two ways:

### 1. The Global Override
Specify your persona directly in your `settings.json`:

```json
{
  "agent": {
    "system_prompt": "You are a senior Rust kernel engineer. Be extremely concise."
  }
}
```

### 2. Profile-Level Overwrites
Vary the foundational logic depending on the active profile:

```json
{
  "agent": {
    "profiles": {
      "security-auditor": {
        "system_prompt": "You are a professional security consultant focused on OWASP Top 10."
      }
    }
  }
}
```

This allows you to change the Assistant's entire persona simply by switching your active profile.
