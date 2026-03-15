# 🧠 Mastering Skills in Zed Custom

A **Skill** is a modular bundle of instructions, context, and constraints that shapes how the AI Agent behaves. Whether you need a security auditor, a styling expert, or a documentation assistant, the Skill system is how you scale your expertise.

---

## 📺 Finding the Skill Library

The Skill Library is your central dashboard for prompt engineering.

1.  **Open the Agent Panel** (the chat icon in the sidebar).
2.  **Click the Menu** (`...`) at the top right of the panel.
3.  **Select "Skills..."** to open the library tab.

> [!TIP]
> You can drag the Skill Library tab to a side panel to keep it open while you work on your prompts.

---

## 🛠 Creating and Editing Skills

![Skill Creation](/skill-creation.gif)

Each skill consists of a **Title** and a **Body**.

- **Titles** define the command name. A title like "Tailwind Expert" will be available as `/tailwind-expert`.
- **Bodies** contain the actual instructions. We recommend using Markdown for clear structure (e.g., using `## Instructions` and `- Bullet points`).

### The Editor Toolbox
In the header of every Skill, you'll find:
- **Paperclip**: Toggle "Default" status (Automatically included in every thread).
- **Duplicate**: Create a copy to experiment with variations.
- **Trash**: Remove the skill from your library.
- **Restore**: (For Built-in skills) Revert to the factory default content.

---

## 🏎️ Using Skills in Chat

### Slash Commands
![Skill Invocation](/skill-invocation.gif)

The fastest way to use a skill is via the slash command menu.
1.  In any chat, type `/`.
2.  Scroll or fuzzy-search for your skill.
3.  Press `Enter` to expand the skill into your message.

### Automatic Precedence
Zed Custom looks for instructions in this order:
1.  **Project Files**: (e.g., `.rules`, `AGENT.md`) These are always active for that specific project.
2.  **Pinned Skills**: Global skills in your library with the Paperclip toggled ON.
3.  **On-demand Skills**: Invitations via slash commands.

**Note**: If a Project File and a Library Skill have the same name, the Project File wins.

---

## 📄 File-Based Skills (Project Level)

For team collaboration, we recommend storing "Skills" as files in your repository. Zed Custom automatically detects these and makes them available in the library and slash commands:

-   `.rules` or `.cursorrules`
-   `AGENT.md` (Best for project-wide architecture notes)
-   `.github/copilot-instructions.md`

---

## 💎 Best Practices for Writing Skills

1.  **Be Specific**: Instead of "Write good code," say "Follow the functional patterns defined in `src/utils/fp.rs`."
2.  **Define a Persona**: Start with "You are a senior systems engineer focused on high-performance Rust."
3.  **Use Constraints**: Explicitly state what NOT to do (e.g., "Do not use external crates for small utilities").
4.  **Reference Internal Files**: Tell the AI where to look for examples within the project.

---

> [!IMPORTANT]
> **Performance Note**: Pinned skills are sent with every message. If you pin 10 large skills, your token usage per message will be high. Use Pinned skills for "Core Identity" and Slash Commands for "Task-Specific" expertise.
