# 🧠 Agent Long-Term Memory System

The most significant upgrade to this Zed fork is the deeply integrated, persistent SQLite memory database. Most AI coding assistants suffer from "amnesia" between editor restarts, forcing developers to re-explain their architectural constraints, internal libraries, and known issues every single day. 

We solved this.

## How it works

The agent uses two primitive tools: `@remember` and `@recall`.

### 1. The `@remember` Tool
When you make a significant design decision, setup a new internal API, or figure out a tricky bug, you can tell the agent to remember it. The agent will categorize the memory (e.g., *Architecture*, *Patterns*, *Issues*, *Procedures*, or *Notes*) and record it in a local database scoped entirely to your current project. 

**Practical Example:**
> **You:** "We just decided that all database mutations must be routed through the `MutationManager` service. Never modify the database directly. Use @remember to store this."
> 
> **Agent:** *Uses the `remember` tool.* "I have stored this pattern under Architecture for this project. I will ensure future mutations go through the `MutationManager`."

![Remember Tool Demo](/demo-remember.gif)

### 2. The `@recall` Tool
When you start a new session, you don't need to feed it 50 files of context. Simply ask a question, and the agent will use the `@recall` tool to retrieve exactly what you saved previously—whether it was yesterday or six months ago.

**Practical Example:**
> **You:** *(Next day, new session)* "Can you write a new function to update the user's email?"
> 
> **Agent:** *Uses the `recall` tool, retrieves the `MutationManager` memory.* "Certainly! Based on this project's architecture, I see we need to route this through the `MutationManager` service rather than updating the DB directly. Here is the implementation..."

![Recall Tool Demo](/demo-recall.gif)

### Privacy First
Because this is built on SQLite directly into the Zed binary, your memories are stored purely on your local file system (`~/.local/share/zed/...`). No embeddings are sent to third parties for storage, and you retain 100% control over the context database.
