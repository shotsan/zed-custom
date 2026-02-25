# 📝 Custom Rule Insertion

A major pain point with standard AI coding assistants is their inability to consistently follow a team's specific coding guidelines or architectural constraints. 

To solve this, our Zed agent natively supports **Custom Rule Insertion** via `.rules` files.

## How it works

The agent's context engine looks for specific rule files at the root of your project workspace and automatically injects their contents directly into the AI's foundational System Prompt.

Currently, the engine scans the workspace for:
- `.rules` (General project guidelines)
- `.cpp_rules` (C++ specific guidelines)
- `.python_rules` (Python specific guidelines)

If any of these files exist, the template engine (`crates/agent/src/templates.rs`) dynamically attaches them to the prompt instructions.

### Example `.rules` file:
```md
# General Rules
- Always use async/await for I/O operations.
- Do not use `unwrap()`, handle errors explicitly using `anyhow`.
- Prefer absolute imports over relative imports.
- Make sure to add trace logging to any new public methods.
```

### Example `.python_rules` file:
```md
# Python Rules
- We strictly use `pydantic` for data validation, do not use standard dataclasses.
- Type hints are strictly required for all function signatures.
```

## Why this is powerful

By committing these `.rules` files directly into your repository, *every developer on your team* who uses this custom Zed build will automatically have an AI assistant that obeys your specific codebase architecture and styling guidelines—without requiring them to manually configure complex custom instructions.
