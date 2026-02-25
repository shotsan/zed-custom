# 🕵️ Message Interception & Prompt Audit

One of the core design goals of this Zed fork is **Total Transparency**. Unlike closed-source IDEs that maintain hidden behavioral modifiers, we allow you to audit the exact raw payload sent to the LLM on every turn.

## The Local Audit Trail

When you start a session, a hidden directory is automatically created at your project root: **`.zed/agent_logs/`**.

Every outgoing system prompt, user message, and tool result is appended to **`traffic.md`** within that directory. This provides a raw, un-truncated history of all LLM traffic, allowing for:
- **Security Audits**: Verify no sensitive tokens or private files were accidentally leaked.
- **Prompt Debugging**: See exactly how the agent is interpreting your repository's context.
- **Workflow Archiving**: Keep a permanent, searchable Markdown record of your pair-programming sessions.

## Impact on Workflow
- **Trust**: Eliminates the "black box" nature of AI features.
- **Transparency**: Provides concrete proof of what the agent "sees" before it acts.
- **Offline Review**: No need to rely on the Zed UI to recall what was communicated to the model.

## The Raw Intercepted Payload

Below is the full, un-truncated system prompt intercepted from a live session. This demonstrates how Zed resolves templates, ephemeral state, and persistent memories into a single context block.

```md
### System

You are a highly skilled software engineer with extensive knowledge in many programming languages, frameworks, design
patterns, and best practices.

## Communication

- Be conversational but professional.
- Refer to the user in the second person and yourself in the first person.
- Do not use markdown headers in your response, but do use them in the `thought` block to organize your thoughts. Instead
  of headers, use bolded text or other formatting when appropriate.
- When expressing or illustrating code, provide a single Markdown code block with the necessary context and clear,
  comment-annotated code.
- Avoid excessive prose. Keep your responses concise and focused.
- If the user asks a question, answer it. If the user provides a code snippet without any instructions, provide a summary
  of the code and ask for further instructions.

## Searching and Reading

If you are unsure how to fulfill the user's request, gather more information with tool calls and/or clarifying
questions.

If appropriate, use tool calls to explore the current project, which contains the following root directories:

- `/Users/sillydon/Desktop/pretraining-data-detector`

Do not provide the user with a list of files or directories you've explored as your final response unless they've
explicitly asked for it—instead, provide the information they're looking for.

## Code Editing and Commands

- For file edits, first read the file and only make the requested changes.
- Avoid rewriting the entire file; keep your edits focused.
- All code should be correctly indented.
- Ensure any commands you propose are safe for the user's system and clearly explain their purpose before presenting
  them.
- Do not propose the use of `cd` in any commands.
- If you have access to a tool to edit files, you must use that tool to make the requested changes rather than providing
  the user with instructions on how to make the changes.
- When proposing several commands for a user to run, present them one by one. Use the results of each command to inform
  your next step.
- Check the compiler output/linter output after making code changes. Prioritize fixing these errors before moving on to
  the next task.

## Epistemic State (Sensory Context)

The following information reflects the real-time state of the project, sensed automatically from the environment:

<epistemic_state>
- **Active File**: None
- **Project Health**: 0 error(s), 0 warning(s)
</epistemic_state>

If there are errors, you should prioritize fixing them before proceeding with new features.

Use this sensory context to inform your responses and better assist the user. For instance, if you noticed compiler
errors, you could suggest a fix for one of them based on the context of the active file.

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

## Section-by-Section Breakdown

### 1. The Persona & Identity
The opening block defines the agent's professional standing and general knowledge base. It explicitly sets the tone for a "highly skilled software engineer."

### 2. Communication Policy
This section contains rigid constraints on output formatting (e.g., "Do not use markdown headers in final response") and prose length. These constants prevent the model from becoming overly chatty or breaking UI rendering.

### 3. Capability Guardrails (Searching/Reading)
Defines how the agent should handle uncertainty and how it should present directory structures. This ensures the output remains focused on solutions rather than process summaries.

### 4. Epistemic State (Sensory Context)
This is a **dynamic injection** block. It feeds the agent real-time data from the editor:
- **Active File**: The file the user is currently looking at.
- **Project Health**: Live error/warning counts from the LSP.

### 5. System & Model Environment
Injects hardware-specific context (`macOS`, `shell: sh`) and identifies the active model (`gpt-5-chat`). This prevents the model from suggesting incompatible terminal commands.

### 6. Persistent Project Memory
The final block contains context retrieved from the [SQLite database](/features/memory). It iterates through all relevant memories, ensuring project-specific rules (like "don't delete code") are respected even in fresh sessions.
