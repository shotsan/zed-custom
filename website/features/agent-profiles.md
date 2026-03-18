# 👤 Agent Profiles

Zed Custom includes powerful **Agent Profiles**, a highly requested feature that allows you to configure specific environments, capabilities, and system prompts tailored for different tasks.

## Managing Profiles

In the assistant panel, you can now view a list of default profiles and any custom profiles you create. You can switch between profiles, creating individual threads where the chosen profile has full context and capability for your specialized task.

To access the profile settings, click on the **Settings (gear)** icon inside the Assistant panel. Or, click the profile dropdown at the top of an Assistant thread to quickly select a profile.

You'll find your different Agent Profiles populated right in your `settings.json`, and can be manipulated via GUI natively inside Zed.

## What is configurable?

You can configure several options inside every individual Agent Profile to give it a unique set of abilities and behaviors.

### 1. Model Selection
Assign a specific Default LLM to a given agent profile. Do you need a coding profile that defaults to Claude 3.7 Sonnet? Or a rapid question-answering profile that defaults to Claude 3.5 Haiku? Pick it once and the Agent Profile will always start with it.

### 2. Available Tools
You can selectively enable or disable tools for each profile. For instance, you could configure a "Web Researcher" profile that only has access to the `/search` and `/fetch` tools, so it doesn't accidentally attempt to read or write local project files.

### 3. Context Servers
Integrate with different MCP (Model Context Protocol) servers on a per-profile basis! If you have multiple custom servers available, you can turn them on specifically for an Agent Profile designed simply to query your company's API.

### 4. Custom Instructions & System Prompts
You can specify:
- **Instructions**: Snippets of rules or behavioral adjustments appended to the base Zed system prompt (for example: "Always use TailwindCSS").
- **System Prompts**: Complete overwrites of the default Zed AI system prompt. Build an entirely distinct character or specialized engineer persona.

## Configuring profiles via UI vs Settings

You can use the new Configuration Modal inside the Assistant panel to modify your current profile, create a new profile from an existing one, or start from scratch! 

Changes executed via the UI are instantly serialized into your workspace's `settings.json` file. 
