# 🧠 Tool: `@remember`

The `@remember` tool allows the AI agent to persist critical project context into a long-term SQLite database, ensuring important decisions or patterns are not lost between sessions.

## Technical Implementation

When the agent invokes `@remember`, the system executes an `INSERT OR REPLACE` SQL statement against the local `memories` table (located in `~/.local/share/zed/`). 

The implementation in `crates/agent/src/tools/memory_tools.rs` enforces categorization into one of five schemas:
1.  **Architecture**: structural rules.
2.  **Patterns**: code styles.
3.  **Issues**: bug resolutions.
4.  **Procedures**: build/deploy steps.
5.  **Notes**: general facts.

## Usage Examples

### Example 1: Preserving Architectural Decisions
<ZedChat>
  <template #user>
    Note that this project uses Rust 1.76 specifically for this workspace. Never use 1.78+ features.
  </template>
  <template #assistant>
    Persisting tool-chain constraint to memory...
  </template>
  <template #output>
    @remember({
      category: "Procedures",
      content: "Workspace strictly uses Rust 1.76. Do not use 1.78+ features."
    })
  </template>
</ZedChat>

### Example 2: Documenting Legacy Debt
<ZedChat>
  <template #user>
    The `legacy_token` module in `crates/auth` is deprecated. Always use `SessionManager` instead.
  </template>
  <template #assistant>
    Recording deprecation policy...
  </template>
  <template #output>
    @remember({
      category: "Architecture",
      content: "`legacy_token` is deprecated. Use `SessionManager` for all new auth code."
    })
  </template>
</ZedChat>

## Workflow Impact
- **Institutional Memory**: The agent remembers *why* you chose a specific pattern, even months after the conversation.
- **Rule Enforcement**: Injects these memories directly into the agent's subconscious prompt for every future turn.
- **Onboarding Speed**: Quickly catches the agent up on obscure project constraints that aren't in the global LLM training data.
- **Scalable Context**: Unlike `.rules` files, memories are retrieved contextually, saving tokens while maintaining precision.
