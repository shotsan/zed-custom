# 🎭 AI Chat Personas

AI Chat Personas allow you to dynamically adjust the tone and personality of the Assistant directly from the Chat Panel. Whether you need a straightforward technical answer or a bit of witty motivation, you can switch moods with a single click.

---

## The Mood Picker

Located in the top-right toolbar of the Agent Panel, the Mood Picker provides instant access to five distinct personas:

1.  **🚀 Excited**: High-energy, motivating, and enthusiastic responses. Perfect for kicking off a new project.
2.  **✨ Witty**: Clever, sometimes sarcastic, and engaging. Great for keeping the development process fun.
3.  **😂 Funny**: Playful and lighthearted. Injects humor into the conversation to reduce frustration.
4.  **😊 Enjoyable**: Pleasant, helpful, and friendly. A balanced and positive companion.
5.  **➖ Straightforward**: Direct, technical, and concise. Best for when you just need the code and nothing else.

---

## How It Works

Selecting a persona dynamically injects specific behavioral instructions into the Assistant's system prompt. This happens in real-time — the model adopts the new tone immediately in its next response.

### Persistent Settings
Your selected mood is saved natively in your `settings.json`, ensuring your preferred persona is preserved across sessions.

```json
{
  "agent": {
    "persona": "witty"
  }
}
```

---

## Visual Indicators

Each persona is represented by a high-quality, expressive icon in the toolbar:

-   **Straightforward**: A clean dash.
-   **Witty**: Magical sparkles.
-   **Funny**: A laughing face.
-   **Enjoyable**: A friendly smile.
-   **Excited**: A rocket ship.

---

## Workflow Integration

-   **Focus Mode**: Switch to **Straightforward** when you're in deep flow and need minimal text.
-   **Debugging Support**: Use **Funny** or **Enjoyable** to take the edge off a particularly tough bug.
-   **New Feature Hype**: Toggle **Excited** when brainstorming new architecture or celebrating a successful deployment.
