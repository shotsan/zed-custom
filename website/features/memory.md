# 🧠 Long-Term Project Memory (SQLite)

The Long-Term Memory system solves the "context amnesia" problem common in AI IDEs by persisting architectural decisions, recurring bugs, and project-specific patterns across editor restarts.

## Architectural Overview

Unlike simple `.md` rule files that are always present in the prompt (leading to token bloat), this system uses a **Retrival-Augmented Generation (RAG)** approach backed by a local SQLite database.

Knowledge is structured into five distinct categories to help the agent maintain a clean conceptual map of your project:
1.  **Architecture**: Core structural decisions.
2.  **Patterns**: Recurring code styles or library usage.
3.  **Issues**: Tricky bugs and their historical fixes.
4.  **Procedures**: Setup steps, build commands, and deployment flows.
5.  **Notes**: General project trivia.

## Implementation Details

The core logic resides in `crates/agent/src/tools/memory_tools.rs`. The database is stored locally on your filesystem at `~/.local/share/zed/project_memory.db` (scoped by project path).

- **Fact Ingestion**: The agent uses the `@remember` tool to write new rows using `INSERT OR REPLACE`.
- **Fact Retrieval**: The agent uses the `@recall` tool to execute `SELECT` queries, often triggered by a semantic search query or a category filter.

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

## Comparison: SQLite vs .md Files

| Feature | Standard IDEs (.md) | Zed Custom (SQLite) |
| :--- | :--- | :--- |
| **Token Usage** | Constant / Exponential | Minimal (Selective Retrieval) |
| **Searchability** | Linear Text Scan | Structured SQL/Semantic Search |
| **Scaling** | Fails at large rule-sets | Handles thousands of project facts |

## Workflow Impact
- **Eliminate Regressions**: The agent proactively warns you if you attempt to re-implement a pattern that was previously archived as "problematic."
- **Institutional Knowledge**: New developers joining the project gain an instant "AI mentor" that remembers every architectural pivots and obscure bug fix.
- **Context Management**: Drastically reduces the "Recap Phase" at the start of every session. The agent starts with a warm cache of project-specific facts.
- **Precision Grounding**: Unlike generic LLM training, these memories ensure the agent follows *your* repository's unique conventions.
