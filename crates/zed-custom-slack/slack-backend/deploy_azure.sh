#!/bin/bash
set -e

# Azure configuration variables
RESOURCE_GROUP="zed-custom-slack-rg"
LOCATION="eastus"
APP_LOCATION="eastus2" # Using eastus2 due to eastus capacity issues
ACR_NAME="zedslackacr1781748782"
ENV_NAME="zed-slack-env-eastus2"
REDIS_NAME="zedslackredis"
KV_NAME="zedslackkv$(date +%s)"
APP_NAME="zed-slack-backend"

# Ensure the user has provided Slack credentials in .env or arguments
if [ ! -f ".env" ]; then
    echo "Please create a .env file with your Slack credentials (SLACK_CLIENT_ID, etc.)"
    exit 1
fi
source .env

echo "=========================================================="
echo "Creating Resource Group..."
az group create --name $RESOURCE_GROUP --location $LOCATION

echo "=========================================================="
echo "Skipping ACR build as it is already pushed..."
# az acr create --resource-group $RESOURCE_GROUP --name $ACR_NAME --sku Basic --admin-enabled true
az acr build --registry $ACR_NAME --image $APP_NAME:latest -f Dockerfile .

echo "=========================================================="
echo "Ensuring Azure Cache for Redis exists..."
# az redis create --location $LOCATION --name $REDIS_NAME --resource-group $RESOURCE_GROUP --sku Basic --vm-size c0
REDIS_KEY=$(az redis list-keys --name $REDIS_NAME --resource-group $RESOURCE_GROUP --query primaryKey -o tsv)
REDIS_HOST=$(az redis show --name $REDIS_NAME --resource-group $RESOURCE_GROUP --query hostName -o tsv)
REDIS_URL="redis://:${REDIS_KEY}@${REDIS_HOST}:6379"

echo "=========================================================="
echo "Provisioning Azure Key Vault..."
az keyvault create --name $KV_NAME --resource-group $RESOURCE_GROUP --location $APP_LOCATION --enable-rbac-authorization true

echo "Granting your user account permission to add secrets..."
USER_ID=$(az ad signed-in-user show --query id -o tsv)
az role assignment create --role "Key Vault Secrets Officer" --assignee $USER_ID --scope "/subscriptions/$(az account show --query id -o tsv)/resourceGroups/$RESOURCE_GROUP/providers/Microsoft.KeyVault/vaults/$KV_NAME"

echo "Waiting for role assignment propagation (15 seconds)..."
sleep 15

echo "Storing secrets in Key Vault..."
az keyvault secret set --vault-name $KV_NAME --name "SlackClientId" --value "$SLACK_CLIENT_ID" >/dev/null
az keyvault secret set --vault-name $KV_NAME --name "SlackClientSecret" --value "$SLACK_CLIENT_SECRET" >/dev/null
az keyvault secret set --vault-name $KV_NAME --name "SlackAppToken" --value "$SLACK_APP_TOKEN" >/dev/null
az keyvault secret set --vault-name $KV_NAME --name "CentrifugoApiKey" --value "$CENTRIFUGO_API_KEY" >/dev/null
az keyvault secret set --vault-name $KV_NAME --name "RedisUrl" --value "$REDIS_URL" >/dev/null

echo "=========================================================="
echo "Ensuring Container Apps Environment exists..."
# az containerapp env create --name $ENV_NAME --resource-group $RESOURCE_GROUP --location $LOCATION

echo "=========================================================="
echo "Deploying Axum Backend to Azure Container Apps with Managed Identity & Key Vault Secrets..."
# We create it with system-assigned identity FIRST
az containerapp create \
  --name $APP_NAME \
  --resource-group $RESOURCE_GROUP \
  --environment $ENV_NAME \
  --image "$ACR_NAME.azurecr.io/$APP_NAME:latest" \
  --target-port 8080 \
  --ingress external \
  --registry-server "$ACR_NAME.azurecr.io" \
  --system-assigned

echo "=========================================================="
echo "Granting Container App permission to read secrets from Key Vault..."
PRINCIPAL_ID=$(az containerapp identity show --name $APP_NAME --resource-group $RESOURCE_GROUP --query principalId -o tsv)
az role assignment create --role "Key Vault Secrets User" --assignee $PRINCIPAL_ID --scope "/subscriptions/$(az account show --query id -o tsv)/resourceGroups/$RESOURCE_GROUP/providers/Microsoft.KeyVault/vaults/$KV_NAME"

echo "Waiting for role assignment propagation (15 seconds)..."
sleep 15

echo "=========================================================="
echo "Updating Container App with Key Vault Secrets..."
az containerapp secret set --name $APP_NAME --resource-group $RESOURCE_GROUP \
  --secrets \
    "slack-client-id=keyvaultref:https://$KV_NAME.vault.azure.net/secrets/SlackClientId,identity=system" \
    "slack-client-secret=keyvaultref:https://$KV_NAME.vault.azure.net/secrets/SlackClientSecret,identity=system" \
    "slack-app-token=keyvaultref:https://$KV_NAME.vault.azure.net/secrets/SlackAppToken,identity=system" \
    "centrifugo-api-key=keyvaultref:https://$KV_NAME.vault.azure.net/secrets/CentrifugoApiKey,identity=system" \
    "redis-url=keyvaultref:https://$KV_NAME.vault.azure.net/secrets/RedisUrl,identity=system"

echo "Updating Container App Environment Variables to reference the new secrets..."
az containerapp update \
  --name $APP_NAME \
  --resource-group $RESOURCE_GROUP \
  --set-env-vars \
    SLACK_CLIENT_ID="secretref:slack-client-id" \
    SLACK_CLIENT_SECRET="secretref:slack-client-secret" \
    SLACK_APP_TOKEN="secretref:slack-app-token" \
    CENTRIFUGO_API_KEY="secretref:centrifugo-api-key" \
    REDIS_URL="secretref:redis-url"

echo "=========================================================="
echo "Deployment complete! Fetching the FQDN..."
FQDN=$(az containerapp show --name $APP_NAME --resource-group $RESOURCE_GROUP --query properties.configuration.ingress.fqdn -o tsv)

echo ""
echo "=========================================================="
echo "Backend securely deployed with Azure Key Vault!"
echo "Your API URL is: https://$FQDN"
echo "Set your Slack App Redirect URI to: https://$FQDN/slack/oauth_redirect"
echo "Your users can now visit: https://$FQDN/slack/install"
echo "=========================================================="
