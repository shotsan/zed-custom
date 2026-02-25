# 🧠 Long-Term Memory (SQLite)

This build features a persistent SQLite database that stores project context across editor restarts, avoiding "context amnesia."

## Core Workflow

### Step 1: Fact Ingestion (`@remember`)
When the agent learns something new (e.g., an architectural rule), it records it in the local DB.

<ZedChat>
  <template #user>
    Always use the "Service Layer" for DB queries.
  </template>
  <template #assistant>
    Stored as an architectural rule.
  </template>
  <template #output>
    @remember({ category: "Architecture", content: "Use Service Layer for DB queries." })
  </template>
</ZedChat>

### Step 2: Context Retrieval (`@recall`)
In future sessions, the agent queries the DB to maintain consistency.

<ZedChat>
  <template #user>
    How should I write this new query?
  </template>
  <template #assistant>
    Recalling database patterns...
  </template>
  <template #output>
    @recall({ query: "database pattern" })
    >> Found: "Use Service Layer for DB queries."
  </template>
</ZedChat>

## Key Advantages

| Feature | standard `.md` Rules | Zed SQLite Memory |
| :--- | :--- | :--- |
| **Storage** | Single file | Structured SQL DB |
| **Retrieval** | Global (Bloat) | Selective (@recall) |
| **Evolution** | Manual | Organic (@remember) |
