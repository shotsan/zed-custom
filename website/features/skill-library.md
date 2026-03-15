# 📚 Skill Library

The **Skill Library** is a centralized repository for managing AI "Expertise." It allows you to save and organize modular system prompts that define specific coding styles or architectural patterns.

> [!NOTE]
> **Current Status**: The management UI and database storage are fully functional. Direct invocation via custom slash commands (e.g., typing `/my-skill`) is currently in development.

## Skill Categories

Skills are currently organized into these tiers:

1.  **Project Skills**: Derived automatically from files in your workspace (e.g., `.rules`, `.cursorrules`, `AGENT.md`). These are "transient" and stay with the repository.
2.  **Global Skills (My Skills)**: Your personal library stored in a local SQLite database (`prompts-library-db`). These persist across all your projects.
3.  **Default Skills**: Any global skill you flag as "Default" will be automatically attached to every new AI thread.
4.  **Built-in Skills**: High-quality templates bundled with Zed Custom (currently including "Commit Message").

## How to use Skills (Current)

### 1. Saving Expertise
- Click the **"Save This as a Skill"** button in any chat thread to archive a particularly good prompt.
- These are stored in your local library and can be edited later.

### 2. Attaching to a Thread (The Paperclip)
- Open the **Skill Library** window (`cmd-shift-s` or via the menu).
- Use the **Paperclip icon** to "Pin" a skill to your current conversation.
- **Permanent Attachment**: Once pinned, these instructions are appended to the system prompt for *every* message in that thread.

### 3. One-off Insertion (The `/prompt` command)
- If you don't want to permanently attach a skill, you can use the built-in `/prompt` command in the chat.
- Type `/prompt` followed by the name of your skill to insert its contents directly into your message.

---

## Planned Features (v0.3.x)

- **Direct Slash Commands**: Registering every custom skill as a top-level command (e.g., `/rust-expert`) so you don't have to type `/prompt` first.
- **Automatic Expert Library**: Expanding the "Built-in" section with specialized agents for Rust, UI design, and Security auditing.

---

## Technical Details

- **Storage**: Global skills are stored in a local Heed/SQLite database at `~/.config/zed-custom/prompts/prompts-library-db.0.mdb`.
- **Precedence**: If a Project Skill and a Global Skill share the same name, the **Project Skill** takes precedence.
- **Token Impact**: Skills are injected into the system prompt. Using multiple large skills will increase token usage per message.

