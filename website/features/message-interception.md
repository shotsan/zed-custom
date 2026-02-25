# 🕵️ Message Interception (Transparency)

Unlike closed-source IDEs, this fork allows you to audit the exact raw payload sent to the LLM.

## Auditing the AI's Context

By intercepting outgoing requests, you can verify exactly what the AI "knows" about your project before it replies.

### Captured Payload Breakdown

1.  **System Prompt**: The resolved Handlebars template.
2.  **Sensory Context**: Real-time error counts and active files.
3.  **Project Memory**: Injected facts from your SQLite DB.

<ZedChat>
  <template #user>
    Outgoing Intercepted JSON
  </template>
  <template #output>
    {
      "model": "gpt-5-chat",
      "system": "You are a proactive 10x engineer...",
      "messages": [
        { "role": "user", "content": "How do I fix this?" }
      ],
      "project_memory": [
        { "category": "Architecture", "content": "..." }
      ]
    }
  </template>
</ZedChat>

## Why it matters
- **Privacy Audit**: Verify that no sensitive `.env` files or secrets are being leaked.
- **Prompt Debugging**: See exactly how your `.rules` files affect the agent's behavior.
- **Transparency**: No hidden behavioral modifiers or sneaky usage tracking.
