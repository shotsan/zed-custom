# Agent Panel

The Agent Panel allows you to interact with many LLMs and coding agents that can help with various types of tasks, such as generating code, codebase understanding, and other general inquiries like writing emails, documentation, and more.

To open it, use the `agent: new thread` action in [the Command Palette](../getting-started.md#command-palette) or click the ✨ (sparkles) icon in the status bar.

## Getting Started {#getting-started}

If you're using the Agent Panel for the first time, you need to have at least one LLM provider or external agent configured.
You can do that by:

1. [subscribing to our Pro plan](https://zed.dev/pricing), so you have access to our hosted models
2. [using your own API keys](./llm-providers.md#use-your-own-keys), either from model providers like Anthropic or model gateways like OpenRouter.
3. using an [external agent](./external-agents.md) like [Gemini CLI](./external-agents.md#gemini-cli) or [Claude Code](./external-agents.md#claude-code)

## Overview {#overview}

With an LLM provider or an external agent configured, type at the message editor and hit `enter` to submit your prompt.
If you need extra room to type, you can expand the message editor with {#kb agent::ExpandMessageEditor}.

You should start to see the responses stream in with indications of [which tools](./tools.md) the model is using to fulfill your prompt.
From this point on, you can interact with the many supported features outlined below.

> Note that for external agents, like [Gemini CLI](./external-agents.md#gemini-cli) or [Claude Code](./external-agents.md#claude-code), some of the features outlined below may _not_ be supported—for example, _restoring threads from history_, _checkpoints_, _token usage display_, and others. Their availability varies depending on the agent.

### Creating New Threads {#new-thread}

By default, the Agent Panel uses zed-custom's first-party agent.

To choose another agent, go to the plus button in the top-right of the Agent Panel and pick either one of the [external agents](./external-agents.md) installed out of the box or a new [Text Thread](./text-threads.md).

### Editing Messages {#editing-messages}

Any message that you send to the model is editable.
You can click on the card that contains your message and re-submit it with an adjusted prompt and/or new pieces of context.

### Queueing Messages

Messages sent while the agent is in the generating state get, by default, queued.

For the zed-custom agent, queued messages get sent at the next turn boundary, which is usually between a tool call and a response, whereas for external agents, the message gets sent at the end of the generation.

You can edit or remove (an individual or all) queued messages.
You can also still interrupt the agent immediately if you want by either clicking on the stop button or by clicking the "Send Now" (double-enter) on a queued message.

### Checkpoints {#checkpoints}

Every time the model performs an edit, you should see a "Restore Checkpoint" button at the top of your message, allowing you to return your code base to the state it was in prior to that message.

The checkpoint button appears even if you interrupt the thread midway through an edit, as this is likely a moment when you've identified that the agent is not heading in the right direction and you want to revert back.

### Navigating History {#navigating-history}

To quickly navigate through recently updated threads, use the {#kb agent::ToggleNavigationMenu} binding when focused on the panel's editor, or click the menu icon button at the top right of the panel.
Doing that will open a dropdown that shows you your six most recently updated threads.

To view all historical conversations, reach for the `View All` option from within the same menu or via the {#kb agent::OpenHistory} binding.

### Following the Agent {#following-the-agent}

zed-custom is built with collaboration natively integrated, and this naturally extends to collaboration with AI models.
To follow the agent as it reads and edits in your codebase, click on the "crosshair" icon button at the bottom left of the panel.

You can also do that with the keyboard by holding down the `cmd`/`ctrl` modifier when submitting a message.

### Get Notified {#get-notified}

If you send a prompt to the Agent and then put zed-custom in the background, you can choose to be notified when its generation wraps up via:

- a visual notification that appears in the top right of your screen
- a sound notification

These notifications can be used together or individually, and you can use the `agent.notify_when_agent_waiting` and `agent.play_sound_when_agent_done` settings keys to customize that, including turning both off entirely.

### Reviewing Changes {#reviewing-changes}

Once the agent has made changes to your project, the panel will surface which files, how many of them, and how many lines have been edited.

To see which files specifically have been edited, expand the accordion bar that shows up right above the message editor or click the `Review Changes` button ({#kb agent::OpenAgentDiff}), which opens a special multi-buffer tab with all changes.

You can accept or reject each individual change hunk, or the whole set of changes made by the agent.

Edit diffs also appear in singleton buffers.
If your active tab had edits made by the AI, you'll see diffs with the same accept/reject controls as in the multi-buffer.
You can turn this off, though, through the `agent.single_file_review` setting.

## Adding Context {#adding-context}

Although zed-custom's agent is very efficient at reading through your code base to autonomously pick up relevant context, manually adding whatever would be useful to fulfill your prompt is still encouraged as a way to not only improve the AI's response quality but also to speed up its response time.

In zed-custom's Agent Panel, all pieces of context are added as mentions in the panel's message editor.
You can type `@` to mention files, directories, symbols, previous threads, rules files, and diagnostics.

Copying images and pasting them in the panel's message editor is also supported.

When you paste multi-line code selections copied from a buffer, zed-custom automatically formats them as @-mentions with the file context.
To paste content without this automatic formatting, use {#kb agent::PasteRaw} to paste raw text directly.

### Selection as Context

Additionally, you can also select text in a buffer and add it as context by using the {#kb agent::AddSelectionToThread} keybinding, running the {#action agent::AddSelectionToThread} action, or choosing the "Selection" item in the `@` menu.

## Custom System Instructions {#custom-system-instructions}

You can inject thread-specific instructions into the AI's system prompt to set a particular persona or enforce temporary constraints for the current session.

To set these instructions:
1. Click the ✨ (sparkle) icon in the message editor toolbar.
2. Enter your instructions in the modal (e.g., "Always use TypeScript", "Explain code like I'm five").
3. Click "Apply Instructions".

The ✨ icon will turn accent-colored to indicate that custom instructions are active. These instructions are personal to the current thread and will not affect other threads or your project-wide `.rules` files.

## Token Usage {#token-usage}

zed-custom surfaces how many tokens you are consuming for your currently active thread near the profile selector in the panel's message editor.

Once you approach the model's context window, a banner appears above the message editor suggesting to start a new thread with the current one summarized and added as context.
You can also do this at any time with an ongoing thread via the "Agent Options" menu on the top right, where you'll see a "New from Summary" button, as well as simply @-mentioning a past thread in a new one..

### Changing Models {#changing-models}

After you've configured your LLM providers—either via [a custom API key](./llm-providers.md) or through [zed-custom's hosted models](./models.md)—you can switch between their models by clicking on the model selector on the message editor or by using the {#kb agent::ToggleModelSelector} keybinding.

> The same model can be offered via multiple providers - for example, Claude Sonnet 4 is available via zed-custom Pro, OpenRouter, Anthropic directly, and more.
> Make sure you've selected the correct model **_provider_** for the model you'd like to use, delineated by the logo to the left of the model in the model selector.

### Prompt Caching {#prompt-caching}

For model providers that support it (like Anthropic), you can toggle prompt caching on and off using the **Zap** icon button in the thread controls area (at the bottom of the panel).

Prompt caching helps reduce latency and costs for long-running conversations by reusing previously processed context. zed-custom's **Smart Caching** strategy ensures that the system prompt, available tools, and the most recent history entries are cached efficiently within the provider's limits.

### Favoriting Models

You can mark specific models as favorites either through the model selector, by clicking on the star icon button that appears as you hover the model, or through your settings via the `agent.favorite_models` settings key.

The great thing about favoriting models is that you can cycle through them with {#kb agent::CycleFavoriteModels} without opening the model selector, enabling quick experimentation with the models you're already most comfortable with.

## Using Tools {#using-tools}

The new Agent Panel supports tool calling, which enables agentic editing.
zed-custom comes with [several built-in tools](./tools.md) that allow models to perform tasks such as searching through your codebase, editing files, running commands, and others.

You can also extend the set of available tools via [MCP Servers](./mcp.md).

### Profiles {#profiles}

Profiles act as a way to group tools.
zed-custom offers three built-in profiles and you can create as many custom ones as you want.

#### Built-in Profiles {#built-in-profiles}

- `Write`: A profile with tools to allow the LLM to write to your files and run terminal commands. This one essentially has all built-in tools turned on.
- `Ask`: A profile with read-only tools. Best for asking questions about your code base without the concern of the agent making changes.
- `Minimal`: A profile with no tools. Best for general conversations with the LLM where no knowledge of your code base is necessary.

You can explore the exact tools enabled in each profile by clicking on the profile selector button > `Configure` button > the one you want to check out.

Alternatively, you can also use either the command palette, by running {#action agent::ManageProfiles}, or the keybinding directly, {#kb agent::ManageProfiles}, to have access to the profile management modal.

#### Custom Profiles {#custom-profiles}

You can also create a custom profile through the Agent Profile modal.
From there, you can choose to `Add New Profile` or fork an existing one with a custom name and your preferred set of tools.

It's also possible to override built-in profiles.
In the Agent Profile modal, select a built-in profile, navigate to `Configure Tools`, and rearrange the tools you'd like to keep or remove.

zed-custom will store this profile in your settings using the same profile name as the default you overrode.

All custom profiles can be edited via the UI or by hand under the `agent.profiles` key in your `settings.json` file.

#### Profile Instructions and Overrides (The Hybrid Approach) {#profile-instructions}

zed-custom uses a **Hybrid Prompting Approach** that balances powerful default agentic capabilities with user-defined specialization. When you configure a profile, you have two distinct ways to influence the model's behavior:

1.  **Custom Instructions (Hybrid/Augment)**:
    -   **What it is**: Your text is injected into zed-custom's default system prompt.
    -   **Why this choice**: It allows you to add constraints (e.g., "Always use functional patterns") or personas while keeping all of zed-custom's core intelligence intact. The model still knows how to use tools, read your files, and interpret "Sensors" (like build errors) because the framework remains unchanged.
    -   **Context**: These instructions appear at the end of the prompt to ensure high priority in the model's attention.

2.  **System Prompt Override (The "Nuclear" Option)**:
    -   **What it is**: You replace the *entire* system prompt with your own Handlebars template.
    -   **Why this choice**: For advanced users who want to build entirely new agent experiences. You can discard zed-custom's instructions entirely.
    -   **Note on Capability**: If you override the full prompt, the agent might "lose" its ability to use tools or read context unless you explicitly instruct it how to do so in your custom text. However, we still pass the same data context (Project state, memories, etc.) to your template, so you can still use variables like `{{project.name}}` or `{{#each worktrees}}`.

To configure these:
1. Open the **Manage Profiles** modal ({#kb agent::ManageProfiles}).
2. Select the profile you wish to edit.
3. Click on **Configure Instructions**.
4. Use the top editor for **Custom Instructions** and the bottom editor for a **Full Override**.

**Design Philosophy**: We chose this two-tier system to avoid a "black box" experience. Most users only need the **Hybrid** approach to nudge the agent, while the **Override** ensures zed-custom remains an open platform for researchers and power users.

### Tool Approval

zed-custom's Agent Panel surfaces the `agent.always_allow_tool_actions` setting that, if turned to `false`, will require you to give permission to any editing attempt as well as tool calls coming from MCP servers.

You can change that by setting this key to `true` in either your `settings.json` or via the Agent Panel's settings view.

You can also give more granular permissions through the dropdown that appears in the UI whenever the agent requests authorization to run a tool call.

### Model Support {#model-support}

Tool calling needs to be individually supported by each model and model provider.
Therefore, despite the presence of tools, some models may not have the ability to pick them up yet in zed-custom.
You should see a "No tools" label if you select a model that falls into this case.

All [zed-custom's hosted models](./models.md) support tool calling out-of-the-box.

### MCP Servers {#mcp-servers}

Similarly to the built-in tools, some models may not support all tools included in a given MCP Server.
zed-custom's UI will inform you about this via a warning icon that appears close to the model selector.

## Text Threads {#text-threads}

["Text Threads"](./text-threads.md) present your conversation with the LLM in a different format—as raw text.
With text threads, you have full control over the conversation data.
You can remove and edit responses from the LLM, swap roles, and include more context earlier in the conversation.

For users who have been with us for some time, you'll notice that text threads are our original assistant panel—users love it for the control it offers.
We do not plan to deprecate text threads, but it should be noted that if you want the AI to write to your code base autonomously, that's only available in the newer, and now default, "Threads".

## Errors and Debugging {#errors-and-debugging}

In case of any error or strange LLM response behavior, the best way to help the zed-custom team debug is by reaching for the `agent: open thread as markdown` action and attaching that data as part of your issue on GitHub.

You can also open threads as Markdown by clicking on the file icon button, to the right of the thumbs down button, when focused on the panel's editor.

## Feedback {#feedback}

zed-custom supports rating responses from the agent for feedback and improvement.

> Note that rating responses will send your data related to that response to zed-custom's servers.
> See [AI Improvement](./ai-improvement.md) and [Privacy and Security](./privacy-and-security.md) for more information about zed-custom's approach to AI improvement, privacy, and security.
> **_If you don't want data persisted on zed-custom's servers, don't rate_**. We will not collect data for improving our Agentic offering without you explicitly rating responses.

The best way you can help influence the next change to zed-custom's system prompt and tools is by rating the LLM's response via the thumbs up/down buttons at the end of every response. In case of a thumbs down, a new text area will show up where you can add more specifics about what happened.

You can provide feedback on the thread at any point after the agent responds, and multiple times within the same thread.
