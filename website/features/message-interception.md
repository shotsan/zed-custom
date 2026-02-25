# 🕵️ Full Message Interception

When you use a proprietary or closed-source AI IDE (like Cursor, Windsurf, or GitHub Copilot), you are completely blind to the telemetry and context being sent to the LLM behind the scenes. You don't know if they are silently uploading your `.env` variables, injecting arbitrary context, or modifying your prompt.

**This Custom Zed Fork operates on total transparency.**

Because we intercept all outgoing LLM network requests locally, we have **Full Message Interception**. We can see the *exact* raw payload (including the dynamic injected System Prompt, the Epistemic State, the active SQLite Memories, and the JSON Tool Schemas) that is transmitted over the wire.

## Real-World Intercepted Prompt Example

Below is a live, real-world example of an outgoing system prompt that was intercepted by our engine. This demonstrates exactly how the Handlebars template (`system_prompt.hbs`) resolves all variables into the final string before it hits the Anthropic/OpenAI API:

```md
### System

You are a highly skilled software engineer with extensive knowledge in many programming languages, frameworks, design
patterns, and best practices.

## Communication

- Be conversational but professional.
- Refer to the user in the second person and yourself in the first person.
...

## Searching and Reading

If you are unsure how to fulfill the user's request, gather more information with tool calls and/or clarifying
questions.

If appropriate, use tool calls to explore the current project, which contains the following root directories:

- `/Users/sillydon/Desktop/pretraining-data-detector`

...

## Epistemic State (Sensory Context)

The following information reflects the real-time state of the project, sensed automatically from the environment:

<epistemic_state>
- **Active File**: None
- **Project Health**: 0 error(s), 0 warning(s)
</epistemic_state>

If there are errors, you should prioritize fixing them before proceeding with new features.

...

## System Information

Operating System: macos
Default Shell: sh

## Model Information

You are powered by the model named gpt-5-chat.

## Project Memory

The following information has been remembered from previous sessions:

### Architecture
Always remember dont delete code at all without my permission
```

### Observation

Notice how the `system_prompt.hbs` template engine seamlessly resolved:
1. The active project path (`/Users/sillydon/Desktop/pretraining-data-detector`).
2. The live diagnostic state (`0 error(s)`).
3. The specific model execution (`gpt-5-chat`).
4. **The SQLite Memory** (*"Always remember dont delete code at all without my permission"*).

With Full Message Interception, you are never guessing what the AI "knows" about your project. You can transparently audit every byte of context.
