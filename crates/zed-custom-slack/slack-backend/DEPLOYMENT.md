# Slack Backend Deployment Guide

This backend is designed to run on Azure Container Apps (ACA) and uses Azure Container Registry (ACR) to store Docker images.

## 1. Updating an Existing Deployment (Fast)

When you make changes to the Rust code in `src/main.rs`, you **should not** run `deploy_azure.sh`. That script is meant for first-time setup and will try to recreate Key Vaults, Redis clusters, and resource groups, which takes a very long time.

Instead, when you want to deploy a quick code update, run the newly created `update_azure.sh` script:

```bash
cd crates/zed-custom-slack/slack-backend
./update_azure.sh
```

**What this does:**
1. Uses `az acr build` to securely package your code and compile the Rust binary in the cloud (which takes a few minutes).
2. Pushes the newly built Docker image (`zed-slack-backend:latest`) into your registry.
3. Uses `az containerapp update` to seamlessly restart your live container app so it pulls the new image without causing downtime.

*(Note: The build phase in ACR typically takes 3–5 minutes because it performs a full release build of the Rust codebase).*

## 2. First-Time Setup (Slow)

If you ever need to deploy to a brand new Azure subscription or region, you can run the full provisioning script:

```bash
cd crates/zed-custom-slack/slack-backend
./deploy_azure.sh
```

**What this does:**
1. Creates the Resource Group, Azure Container Registry, and Azure Cache for Redis.
2. Creates an Azure Key Vault and securely provisions your `.env` secrets into it.
3. Deploys the Container App with a System-Assigned Managed Identity.
4. Grants the Container App permission to pull secrets directly from Key Vault (no secrets are stored in plaintext environment variables).
