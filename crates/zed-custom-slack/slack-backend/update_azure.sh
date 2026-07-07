#!/bin/bash
set -e

ACR_NAME="zedslackacr1781748782"
APP_NAME="zed-slack-backend"
RESOURCE_GROUP="zed-custom-slack-rg"

echo "Building and pushing new Docker image to Azure Container Registry..."
az acr build --registry $ACR_NAME --image $APP_NAME:latest -f Dockerfile .

echo "Updating Azure Container App to use the new image..."
az containerapp update \
  --name $APP_NAME \
  --resource-group $RESOURCE_GROUP \
  --image "$ACR_NAME.azurecr.io/$APP_NAME:latest"

echo "Backend update complete!"
