# 📚 Skill Library

The **Skill Library** is a centralized repository for managing AI "Expertise." It allows you to save, organize, and reuse modular system prompts that define specific coding styles, architectural patterns, or specialized knowledge.

Unlike a static system prompt, the Skill Library is dynamic—allowing you to attach specific expertise to a conversation on the fly.

## Why use the Skill Library?

Standard "Custom Rules" (`.rules` files) are great for project-wide standards, but often you need something more specific or temporary:
- **Architectural Reference**: "Always use the Repository pattern with these specific traits."
- **Testing Standards**: "Write tests using PropCheck and ensure 100% coverage of error branches."
- **Library Expert**: "You are an expert in GPUI's element tree. Focus on performance and layout debugging."

## Skill Categories

Skills are organized into four distinct tiers:

1.  **Project Skills**: Defined in your workspace (via `.rules` or other config). These stay with the repository.
2.  **Default Skills**: Personal skills you've flagged to be **auto-attached** to every new AI thread.
3.  **My Skills**: Your personal library of reusable prompts stored globally across all your projects.
4.  **Built-in Skills**: High-quality "Expert" prompts included out-of-the-box with Zed Custom.

---

## How to use Skills

### 1. Saving a Conversation as a Skill
If you've spent 10 minutes refining a prompt that works perfectly for a specific task, don't let it die with the thread:
- Click the **"Save This as a Skill"** button in the chat interface.
- Give it a name and a description.
- It is now archived in your global library.

### 2. Attaching Skills to a Thread
- Open the **Skill Library** panel.
- Browse or search for the expertise you need.
- Click the **Paperclip** icon to "Pin" it to the current conversation.
- The Agent instantly inherits all the instructions and constraints defined in that skill.

---

## Examples

### Example: "Rust Safety Auditor"
**Title**: Rust Safety Auditor
**Content**:
```text
You are a senior Rust security researcher.
Your primary goal is to find 'unsafe' blocks and ensure they are sound.
1. Check for possible pointer aliasing.
2. Ensure FFI calls handle null pointers.
3. Verify that manual memory management follows the 'Drop' trait strictly.
Always respond with a 'Safety Audit' section first.
```

### Example: "Tailwind UI Expert"
**Title**: Tailwind UI Expert
**Content**:
```text
Exclude all generic CSS. Use only Tailwind utility classes.
Prioritize 'glassmorphism' effects: `bg-white/10 backdrop-blur-md`.
Ensure mobile-first responsiveness for every component.
Use consistent spacing scales (multiples of 4).
```

---

## Technical Details

- **Storage**: Skills are stored in a local SQLite database, ensuring low-latency access and privacy.
- **Token Impact**: Skills are injected directly into the system prompt. Using **[Prompt Caching](./azure-anthropic)** is highly recommended when using multiple large skills simultaneously.
- **Syncing**: Project-level skills are derived from `.rules`, `.cursorrules`, and other known config files in your repository, making them shareable with your team via Git.
