# ☁️ Azure Anthropic & Token Caching

## 1. Zero-Friction Azure Support

If you use enterprise Azure Anthropic endpoints, you're likely familiar with `404 DeploymentNotFound` errors. Standard Zed passes hardcoded string identifiers (like `claude-3-5-sonnet-20241022`) which Azure strictly rejects in favor of exact deployment names.

This fork intercepts these calls natively. If you specify a custom `api_url` in your settings, Zed automatically forwards the exact `serde_name` deployment model you provided, completely eliminating the need for local python proxy servers.

## 2. What is Token Caching?

When you work on massive projects, sending the entire context of your codebase to the LLM on every single turn becomes prohibitively slow and expensive. 

**Anthropic Prompt Caching** attempts to solve this by storing your largest reference files on their servers for roughly 5 minutes. If your next prompt includes the exact same context, Anthropic retrieves the prefix from its RAM instead of recalculating the entire neural net path. This drops multi-minute delays down to less than 2 seconds and drastically cuts your API billing.

### Visualizing the Optimization

Standard Zed hides these metrics deeply within debug menus or disables them outright, making it impossible to know if you are actually saving money or if the cache is thrashing.

In this build, the `show_turn_stats` visualization is **enabled by default**. 

![Azure Anthropic Token Caching Demo](/azure-anthropic-demo.gif)

Whenever you send a message, you immediately see beautifully formatted badges above the feedback buttons:
- **`X cached`**: How many tokens were instantly pulled from Anthropic's server cache.
- **`+X saved`**: The exact number of unused tokens (and cents) you saved on this specific turn. 

No configuration tweaking required. Just pristine, real-time ROI tracking directly in the UI.
