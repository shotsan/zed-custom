# 👤 Agent Profiles

Agent Profiles allow you to define specialized personas for the Assistant, each with its own underlying models, system instructions, and tool capabilities. By hot-swapping profiles, you can change the Assistant's entire technical approach instantly.

---

## Technical Composition

An Agent Profile consists of four core technical layers:

1.  **Model Selection**: Specify which LLM (e.g., `claude-3-5-sonnet`, `gpt-4o`) the Assistant should use as its primary "driver."
2.  **Instructions**: Small snippets of rules or behavioral adjustments appended to the base system prompt (for example: "Always use TailwindCSS").
3.  **System Prompts**: Complete overwrites of the default AI system prompt. Build an entirely distinct character or specialized engineer persona.
4.  **Tool Access**: Toggle granular permissions for every built-in and custom tool (e.g., enabling `deep_research` on some profiles while keeping others restricted).

---

## Where to Configure

Profile definitions are stored natively in your `settings.json`, and can be manipulated via the GUI inside the IDE.

```json
{
  "agent": {
    "profiles": {
      "security-auditor": {
        "model": "claude-3-5-sonnet",
        "instructions": "Always prioritize OWASP Top 10 vulnerabilities in your code audits.",
        "tool_permissions": {
          "terminal": false,
          "deep_research": true
        }
      }
    }
  }
}
```

---

## Workflow Benefits

-   **Expertise Switching**: Use a "Security Auditor" profile for PR reviews and a "Fast Prototyper" for initial feature work.
-   **Context Isolation**: Keep experimental tool configurations isolated from your primary development personas.
-   **Deterministic Behavior**: Fine-tune specific response styles (Technical vs. Narrative) to match your team's internal documentation standards.
