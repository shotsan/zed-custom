# 🧠 Skill Library: The AI's Expertise Engine

The **Skill Library** is the system that allows you to manage and inject specialized instructions into the Assistant. It transforms a generic model into a specialized engineer tailored to your specific stack and conventions.

---

## 🚀 Instant Discovery

Expertise is reachable in seconds.

### 1. Slash Commands
Every skill in your library is automatically registered as a **Slash Command**. Type `/rust` or `/tailwind` directly in the chat to inject specific context.

### 2. Dedicated Management
- **Command Palette**: Search for "Skill Library" or use `cmd-shift-s`.
- **IDE-Grade Editor**: Manage your skills in a dedicated tab with full syntax highlighting and Markdown support.

---

## 📂 Where Skills Live

Skills are managed through two parallel systems: **Global Persistence** and **Project-Level Files**.

### 1. Global Skill Library
These are persistent skills stored in a local database that follow you across all your projects.
- **Pinned Skills**: Click the **Paperclip icon** to keep a skill always active in your system prompt.
- **Expert Templates**: Comes with built-in experts for Rust Security, Tailwind CSS, and Commit Message generation.

### 2. File-Based Project Skills
The system natively supports reading instructions from plain text files in your repository. This is the most direct way to ensure the AI follows specific coding standards for a project.

- **Automatic Detection**: The IDE looks for files like `.rules`, `.cursorrules`, or `AGENT.md` in your project root.
- **Context Priority**: Instructions in these files are automatically prioritized and added to the Assistant's system prompt whenever you are working within that project.
- **Git Compatible**: Because these are just files, you can check them into Git to share coding standards with your team.

---

## 📍 Directory-Level Rules (Advanced Modules)

If you have a large project, you may want different rules for different parts of your code (e.g., a "frontend" vs a "backend").

### 🛠️ Step-by-Step: Creating a Sub-Folder Rule

1. **Navigate to the sub-directory** where you want specific rules (for example, `src/components`).
2. **Create a new file** named `.rules` inside that folder.
3. **Add instructions** specific ONLY to that part of the code. For example:
   ```markdown
   # Component Rules
   - Always use Functional Components.
   - Prefer Tailwind classes over inline styles.
   ```
4. **Save the file.** 

---

## 🌟 Real-World Workflow: Multi-Level Standards

To understand exactly how the Assistant utilizes these files, let’s walk through a project that uses both **Global** and **Scoped** rules.

### Scenario: A Full-Stack Rust & React App

#### Step 1: Set Project-Wide Safety Standards
Create a `.cursorrules` file in your **project root**. This file defines universal rules that the AI must follow regardless of which file you are editing.

1. Create the file: `touch .cursorrules`
2. Add these instructions:
   ```markdown
   # Universal Project Rules
   - Always use explicit error handling.
   - Never use `unwrap()` or `expect()` in production code.
   - Prefer `anyhow` for error propagation.
   ```

#### Step 2: Set Frontend-Specific Styles
Navigate to your `src/frontend/` directory and create a specialized `.rules` file. This tells the AI to use specific libraries and styles only when working in this folder.

1. Create the file: `touch src/frontend/.rules`
2. Add these instructions:
   ```markdown
   # Frontend Rules
   - Use React Functional Components with `export default`.
   - All styling must use Tailwind utility classes.
   - Implement "Glassmorphism" for all card backgrounds.
   ```

#### Step 3: Triggering the Assistant
Now, open a file inside the frontend folder, such as `src/frontend/Header.tsx`, and ask the Assistant to *"Create a login button with a dropdown."*

**The Assistant's Internal Merging Logic**:
- **Directory Path Analysis**: The IDE detects you are editing a file in `src/frontend/`.
- **Root Rule Injection**: It loads the root `.cursorrules`. The Assistant now knows **not** to use `unwrap()`.
- **Local Rule Injection**: It merges the local `src/frontend/.rules`. The Assistant now knows to use **Tailwind** and **Glassmorphism**.
- **Final Output**: The generated code will be a Functional Component using Tailwind classes and safe error handling, automatically aligning with your entire stack's requirements.

---

## ⚙️ Under the Hood: System Prompt Injection

When you start a conversation, the IDE doesn't just "read" your rules—it **injects them directly into the LLM's System Prompt**. 

- **Handlebars Templating**: Our engine uses a high-performance templating system to merge your global pinned skills, project-root rules, and folder-specific `.rules` into a single set of instructions.
- **Pre-Alignment**: This ensures the model is "pre-aligned" with your architectural constraints before it even reads your first message.
- **Auto-Merging**: The final prompt is a composite sum of all rules from the current file's directory up to the root of the worktree.

---

## 🛠️ Managing Your Skills

<div class="features-grid">
  <div class="feature-card">
    <h3>📝 Rich Editor</h3>
    <p>Edit your prompts with full syntax highlighting, auto-indentation, and Markdown support.</p>
  </div>
  <div class="feature-card">
    <h3>🪄 Inline Refinement</h3>
    <p>Highlight part of a skill and use <code>cmd-i</code> to have the AI rewrite or clear up the instructions.</p>
  </div>
  <div class="feature-card">
    <h3>🔗 Pinned Persistence</h3>
    <p>Pinned skills ensure your most important architectural constraints are consistently applied by the model.</p>
  </div>
</div>

---

## ⚡ Real-World Example

If you have a **"Performance Expert"** skill pinned:
> "Always optimize for zero-allocation paths. Prefer SmallVec over Vec for small lists."

The agent will align its code suggestions without you mentioning performance:

<div class="zed-chat-mockup">
  <div class="user-msg">"Write a function to collect 4 string IDs."</div>
  <div class="assistant-msg">
    <small>Consulting <b>Performance Expert</b> skill...</small><br/>
    "Using <code>SmallVec&lt;[String; 4]&gt;</code> to ensure zero heap allocations for this fixed-size list."
  </div>
</div>

---

## 💎 Pro-Tips

-   **Precedence**: Local `.rules` files in a sub-folder take precedence over the root project files for that directory.
-   **Scoping**: Use `AGENT.md` at the project root for high-level technical architecture and `.rules` files for low-level coding style.
-   **Command Aliases**: A skill named "Rust Security Auditor" is available via the `/rust-security-auditor` slash command.
