# 🤖 System Prompts & Persona

A great AI agent is defined by its core instructions. Upstream Zed provides a solid baseline, but to support our custom features (like long-term memory, headless browsing, and tool execution), we had to completely overhaul the system prompt architecture.

## The Prompt Template Engine

The core system prompts are managed in `crates/agent/src/templates.rs` and injected dynamically using Handlebars (`.hbs`) templates.

Our custom `system_prompt.hbs` is broken down into several distinct sections to ensure the LLM strictly adheres to its persona and properly leverages its environment.

### 1. The Persona Block
We instruct the LLM on exactly *who* it is and *how* it should act.
> "You are an expert, proactive Software Engineer serving as an autonomous 10x pair programming partner. You prioritize correct, secure, and highly optimized code..."

### 2. Global Context & Tool Injection
The template engine dynamically lists every custom tool the agent has available (e.g., `remember`, `recall`, `search`, `edit_file`). 

If you are using the Long-Term Memory feature, the engine will query the SQLite database and inject *Project Memories* directly into the system prompt before the first turn even begins!

### 3. Rule Injection
As mentioned in our [Rules Documentation](/features/rules), this is the section where the engine appends contents from `.rules`, `.cpp_rules`, and `.python_rules` directly into the agent's subconscious constraints.

### 4. Behavioral Constraints
We explicitly guide the agent on how to use tools to avoid infinite loops and hallucinations.
> "CRITICAL INSTRUCTION: Before making tool calls, think and explicitly list out any related tools for the task. You must ALWAYS use `grep_search` instead of running `grep` in bash."

## Modifying the Persona

Because the templates are embedded via `rust_embed`, if you want to alter the fundamental persona of the agent, you must:
1. Edit the `crates/agent/src/templates/system_prompt.hbs` file.
2. Recompile the Zed binary (`cargo build --release`).
