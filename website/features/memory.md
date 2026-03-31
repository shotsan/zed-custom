# 🧠 Long-Term Project Memory

The Long-Term Memory system solves the "context amnesia" problem common in AI IDEs by persisting architectural decisions, recurring bugs, and project-specific patterns across editor restarts.

---

## Selective Retrieval

Unlike simple rule files that are always present in every prompt (leading to token bloat), this system utilizes a **Selective Retrieval** approach. It only injects relevant project facts into the context when they are actually needed for your current task.

Knowledge is structured into five distinct categories to ensure high-precision discovery:
1.  **Architecture**: Core structural decisions.
2.  **Patterns**: Recurring code styles or library usage.
3.  **Issues**: Tricky bugs and their historical fixes.
4.  **Procedures**: Setup steps, build commands, and deployment flows.
5.  **Notes**: General project trivia.

---

## 🔍 How to View and Manage Memories

While the Assistant manages these memories autonomously, you have full transparency and can manually review or delete them at any time.

### 1. Opening the Memory Manager
Look for the **Library Icon** in the Assistant panel header. 
- **Visual Hint**: If your project has active memories, a small **accent dot** will appear on the icon, indicating that the Assistant is currently "grounded" in your project rules.

### 2. Reviewing Facts
Clicking the icon opens the **Project Memories & Rules** modal. Here, you can see every fact the Assistant has committed to memory, organized by its category (`Architecture`, `Patterns`, etc.).

### 3. Manual Deletion
If the Assistant learns a pattern that is no longer relevant, or if you want to clear a specific historical issue, simply click the **Trash Icon** next to the memory entry to permanently remove it from the project database.

### 4. Proactive Teaching
Next to the Library icon, you will also find a **Hammer Icon** ("Teach a Rule"). Click this to manually type a rule you want the Assistant to remember for the long-term, ensuring it is committed to the database immediately.

---

## 🛠️ How to Create a Memory (Step-by-Step)

You do not need specialized commands or SQL knowledge. You simply **tell the Assistant** what you want it to remember.

### 1. Give an Instruction
Tell the Assistant about a new rule or pattern for your project.
> **User**: *"From now on, always use the Service Layer pattern for data fetching. Never call the database directly from a React component."*

### 2. The Assistant Commits to Memory
The Assistant recognizes this as a core architectural rule and automatically uses its `@remember` tool:
- **Category**: `Architecture`
- **Content**: *"Strictly use Service Layer for data fetching; no direct DB calls in React components."*

### 3. Verify the Storage
The Assistant will confirm: *"I've stored this as a project architecture memory. I'll ensure future code follows this pattern."*

---

## 🤖 The Autonomous Librarian: How the Model Decides

The Assistant acts as an **Autonomous Librarian** for your project. You do not have to manually tag, sort, or store information—the model handles the logic behind the scenes.

### 🧠 Decision Matrix: When to Remember
The Assistant evaluates every instruction for **Long-Term Utility**. 
-   **Conversation Flow**: *"Can you fix this typo?"* → **IGNORED**. This is a one-off task with no long-term value.
-   **Persistent Rule**: *"We always use snake_case for filenames."* → **COMMITTED**. This is a project-wide pattern that needs to be remembered for future sessions.

### 🏷️ Intelligent Categorization
The Assistant doesn't just "dump" text. It intelligently sorts your instructions into the five categories to ensure high-precision retrieval:
1.  **Architecture**: Core structural rules (e.g., "Use a micro-services backend").
2.  **Patterns**: Coding styles (e.g., "Always use Tailwind utility classes").
3.  **Issues**: Historical bug fixes (e.g., "Fix for the auth race condition").
4.  **Procedures**: Workflow instructions (e.g., "How to run the Docker build").
5.  **Notes**: General project context (e.g., "Testing environment is on port 3001").

---

## Visual Walkthrough

### Step 1: Learning a New Constraint

<ZedChat>
  <template #user>
    We must use the "Service Layer" pattern for all database interactions in this crate. Never call Postgres queries directly from the UI.
  </template>
  <template #assistant>
    Understood. Storing this architectural rule in the project memory.
  </template>
  <template #output>
    @remember({
      category: "Architecture",
      content: "Strictly use Service Layer pattern for DB interactions. UI must not call Postgres directly."
    })
  </template>
</ZedChat>

### Step 2: Recalling Context in a New Session

<ZedChat>
  <template #user>
    I'm ready to add the User Profile page. How should I handle the database part?
  </template>
  <template #assistant>
    Checking project memory for database patterns...
  </template>
  <template #output>
    @recall({ query: "database pattern" })
    >> Found in [Architecture]: "Strictly use Service Layer pattern for DB interactions..."
  </template>
</ZedChat>

---

## Efficiency Comparison

| Feature | Standard Documentation | Long-Term Memory |
| :--- | :--- | :--- |
| **Token Usage** | Constant / Exponential | Minimal (Selective Retrieval) |
| **Searchability** | Linear Text Scan | Semantic & Categorical Search |
| **Scaling** | Fails at large rule-sets | Handles thousands of project facts |

---

## Workflow Impact

- **Eliminate Regressions**: The agent proactively warns you if you attempt to re-implement a pattern that was previously archived as "problematic."
- **Institutional Knowledge**: New developers joining the project gain an instant "AI mentor" that remembers every architectural pivots and obscure bug fix.
- **Context Management**: Drastically reduces the "Recap Phase" at the start of every session. The Assistant starts with a warm cache of project-specific facts.
- **Precision Grounding**: Unlike generic LLM training, these memories ensure the Assistant follows *your* repository's unique conventions.
