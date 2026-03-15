# Using Skills {#using-skills}

A skill is a prompt template that provides instructions or context to the AI agent. Skills can be automatically included in every conversation or invoked on-demand using slash commands.

Currently, zed-custom supports adding skills through files in your project or through the **Skills Library**, which stores skills globally for use across all projects.

## Skills Library {#skills-library}

The Skills Library is a built-in interface for managing your AI instructions. It features a full editor with syntax highlighting and keyboard shortcuts.

### Opening the Skills Library

1. Open the **Agent Panel**.
2. Click on the Agent menu (`...`) in the top right corner.
3. Select **Skills...** from the dropdown.

You can also reach it by running `agent: open skill library` in the command palette or through the `cmd-shift-R` keybinding.

### Invoking Skills via Slash Commands {#slash-commands}

You can invoke any skill from your library directly in the Agent Panel by typing `/` followed by the name of the skill. Skill titles are automatically converted to lowercase "slugs" (e.g., "Rust Safety Auditor" becomes `/rust-safety-auditor`).

When you select a skill completion:
1. The skill's content is expanded into your message editor.
2. You can then add more text or context before sending.

### Pinned Skills (Default Skills) {#pinning-skills}

You can pin a skill using the **Paperclip** icon in the header of the skill editor. 

- **Pinned skills** are automatically included in the context of every new thread.
- **Unpinned skills** are only used when you explicitly invoke them via a slash command or reference them with the `@rule` mention.

### Built-in Expert Skills {#built-in-experts}

zed-custom includes several built-in expert templates that you can use or customize:

- **Rust Safety Auditor**: Focused on identifying potential security issues in unsafe Rust code.
- **Tailwind Expert**: Focused on modern Tailwind CSS best practices, responsiveness, and glassmorphism.
- **Commit message**: A template designed to help generate high-quality git commit messages.

## Project-Level Skills (`.rules`) {#project-rules}

zed-custom also supports project-specific instructions using special files in your worktree. These are automatically included in all interactions within that project.

The following filenames are supported (in order of precedence):
- `.rules`
- `.cursorrules`
- `.windsurfrules`
- `.clinerules`
- `.github/copilot-instructions.md`
- `AGENT.md`
- `AGENTS.md`
- `CLAUDE.md`
- `GEMINI.md`

## Managing Skills {#managing-skills}

Skills in the library can be:
- **Created**: Click the `+` icon in the sidebar.
- **Edited**: Change the content or title directly in the editor.
- **Duplicated**: Click the **Duplicate** icon to create a copy of an existing skill.
- **Deleted**: Click the **Trash** icon to remove a skill.
- **Restored**: Built-in skills can be restored to their default content using the **Restore** icon.

---

> [!TIP]
> You can use the **Inline Assistant** (`cmd-I`) directly within the skill editor to help you write or refine your instructions.
