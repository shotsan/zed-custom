# ☁️ Azure Anthropic & Token Caching

## 1. Zero-Friction Azure Support

If you use enterprise Azure Anthropic endpoints, you're likely familiar with `404 DeploymentNotFound` errors. Standard Zed passes hardcoded string identifiers (like `claude-3-5-sonnet-20241022`) which Azure strictly rejects in favor of exact deployment names.

This fork intercepts these calls natively. If you specify a custom `api_url` in your settings, Zed automatically forwards the exact `serde_name` deployment model you provided, completely eliminating the need for local python proxy servers.

## 2. Token Caching Visualization

Anthropic's Prompt Caching feature is incredibly powerful, dramatically reducing latency and API costs for large contexts. However, standard Zed hides these metrics deeply within debug menus or disables them outright.

In this build, the `show_turn_stats` visualization is **enabled by default**. 

![Azure Anthropic Token Caching Demo](/azure-anthropic-demo.gif)

Whenever you send a message, you immediately see beautifully formatted badges above the feedback buttons:
- **`X cached`**: How many tokens were instantly pulled from the Anthropic cache.
- **`+X saved`**: The exact number of tokens (and money) you saved on this turn. 

No configuration tweaking required. Just pristine, real-time ROI tracking directly in the UI.
