# 🕵️ Message Interception & Prompt Audit

One of the core design goals of this IDE is **Total Transparency**. Unlike closed-source environments that maintain hidden behavioral modifiers, the system allows you to audit the exact raw payload sent to the language model on every turn.

---

## The Local Audit Trail

When you start a session, a hidden directory is automatically created at your project root: **`.agent_logs/`**.

Every outgoing system prompt, user message, and tool result is appended to **`traffic.md`** within that directory. This provides a raw, un-truncated history of all traffic, allowing for:

- **Security Audits**: Verify no sensitive tokens or private files were accidentally included in the prompt context.
- **Behavioral Analysis**: See exactly how the Assistant is interpreting your repository's structure and rules.
- **Workflow Archiving**: Keep a permanent, searchable Markdown record of your pair-programming sessions.

---

## Impact on Workflow

- **Eliminates "Black Box" Logic**: Removes ambiguity about why the Assistant performed a specific action.
- **Concrete Verification**: Provides definitive proof of what the specific model version "saw" before generating its code.
- **Offline Review**: Revisit detailed technical discussions without needing to rely on the active UI thread.

---

## The Raw Intercepted Payload

Below is an example of a system prompt intercepted from a live session. This demonstrates how the IDE resolves templates, ephemeral state (LSP), and persistent project memories into a single context block.

```md
### System

You are a technical software engineer. 
Refer to the user in the second person and yourself in the first person. 
Do not use markdown headers in your response; use bolded text or other formatting.
Provide concise, comment-annotated code blocks only.

## Epistemic State (Sensory Context)

The following information reflects the real-time state of the project, sensed automatically from the environment:

<epistemic_state>
- **Active File**: /src/main.rs
- **Project Health**: 0 error(s), 1 warning(s)
</epistemic_state>

## System Information

Operating System: macOS
Default Shell: zsh

## Model Information

You are powered by the model named claude-3-5-sonnet.

## Project Memory

The following information has been remembered from previous sessions:

### Architecture
Always prioritize functional components and immutable state.
```

---

## Section-by-Section Breakdown

### 1. The Persona & Identity
The instructions define the Assistant's professional tone and formatting constraints, ensuring responses stay concise and UI-friendly.

### 2. Epistemic State (Sensory Context)
This is a **dynamic injection** block. It feeds the Assistant real-time data directly from the IDE:
- **Active File**: The file you are currently editing.
- **Project Health**: Live error/warning counts retrieved from the lsp.

### 3. System & Model Environment
Provides the Assistant with your OS details (`macOS`, `shell: zsh`) and identifies the active model. This prevents the model from suggesting incompatible terminal commands or CLI tools.

### 4. Persistent Project Memory
The final block contains context retrieved from the [Long-Term Memory](/features/memory) database. It ensures project-specific rules (like "prioritize functional components") are respected even in entirely fresh sessions.
