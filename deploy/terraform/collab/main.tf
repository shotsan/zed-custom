terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
    azuread = {
      source  = "hashicorp/azuread"
      version = "~> 2.0"
    }
  }
}

provider "azurerm" {
  features {}
}

provider "azuread" {}

data "azurerm_client_config" "current" {}

data "azurerm_resource_group" "rg" {
  name = "zed-custom-slack-rg"
}

resource "random_string" "suffix" {
  length  = 6
  special = false
  upper   = false
}

resource "random_password" "db_password" {
  length           = 16
  special          = true
  override_special = "!#$%&*()-_=+[]{}<>:?"
}

# Azure AD App Registration for Collab OAuth
resource "azuread_application" "collab" {
  display_name = "Zed Custom Collab Backend"
  
  web {
    redirect_uris = ["https://api.zedcustom.dev/auth/callback"]
    implicit_grant {
      access_token_issuance_enabled = false
      id_token_issuance_enabled     = true
    }
  }
}

resource "azuread_service_principal" "collab" {
  client_id = azuread_application.collab.client_id
}

resource "azuread_application_password" "collab" {
  application_object_id = azuread_application.collab.object_id
}

# Container Registry
resource "azurerm_container_registry" "acr" {
  name                = "zedcollabacr${random_string.suffix.result}"
  resource_group_name = data.azurerm_resource_group.rg.name
  location            = data.azurerm_resource_group.rg.location
  sku                 = "Basic"
  admin_enabled       = true
}

# PostgreSQL Database
resource "azurerm_postgresql_flexible_server" "db_server" {
  name                   = "zed-collab-psql-${random_string.suffix.result}"
  resource_group_name    = data.azurerm_resource_group.rg.name
  location               = "centralindia"
  version                = "15"
  administrator_login    = var.db_admin_user
  administrator_password = random_password.db_password.result
  zone                   = "1"

  storage_mb = 32768
  sku_name   = "B_Standard_B1ms"
}

resource "azurerm_postgresql_flexible_server_database" "collab_db" {
  name      = "collab"
  server_id = azurerm_postgresql_flexible_server.db_server.id
  collation = "en_US.utf8"
  charset   = "utf8"
}

resource "azurerm_postgresql_flexible_server_firewall_rule" "allow_all" {
  name             = "allow-all"
  server_id        = azurerm_postgresql_flexible_server.db_server.id
  start_ip_address = "0.0.0.0"
  end_ip_address   = "255.255.255.255"
}

# Log Analytics Workspace for Container Apps
resource "azurerm_log_analytics_workspace" "law" {
  name                = "zed-collab-law-${random_string.suffix.result}"
  location            = data.azurerm_resource_group.rg.location
  resource_group_name = data.azurerm_resource_group.rg.name
  sku                 = "PerGB2018"
}

# Container App Environment
resource "azurerm_container_app_environment" "env" {
  name                       = "zed-collab-env-${random_string.suffix.result}"
  location                   = data.azurerm_resource_group.rg.location
  resource_group_name        = data.azurerm_resource_group.rg.name
  log_analytics_workspace_id = azurerm_log_analytics_workspace.law.id
}

# Container App
resource "azurerm_container_app" "collab" {
  name                         = "zed-collab-app"
  container_app_environment_id = azurerm_container_app_environment.env.id
  resource_group_name          = data.azurerm_resource_group.rg.name
  revision_mode                = "Single"

  ingress {
    external_enabled = true
    target_port      = 8080
    traffic_weight {
      percentage      = 100
      latest_revision = true
    }
  }

  secret {
    name  = "db-password"
    value = random_password.db_password.result
  }
  
  secret {
    name  = "azure-ad-client-secret"
    value = azuread_application_password.collab.value
  }

  secret {
    name  = "acr-password"
    value = azurerm_container_registry.acr.admin_password
  }

  registry {
    server               = azurerm_container_registry.acr.login_server
    username             = azurerm_container_registry.acr.admin_username
    password_secret_name = "acr-password"
  }

  template {
    container {
      name   = "collab"
      image  = "mcr.microsoft.com/azuredocs/containerapps-helloworld:latest" # Placeholder
      cpu    = 0.5
      memory = "1.0Gi"

      env {
        name  = "DATABASE_URL"
        value = "postgres://${var.db_admin_user}:${random_password.db_password.result}@${azurerm_postgresql_flexible_server.db_server.fqdn}:5432/collab"
      }
      env {
        name  = "AZURE_AD_CLIENT_ID"
        value = azuread_application.collab.client_id
      }
      env {
        name  = "AZURE_AD_CLIENT_SECRET"
        secret_name = "azure-ad-client-secret"
      }
      env {
        name  = "AZURE_AD_TENANT_ID"
        value = data.azurerm_client_config.current.tenant_id
      }
      env {
        name  = "ZED_DOT_DEV_URL"
        value = "https://api.zedcustom.dev"
      }
      env {
        name  = "AZURE_OPENAI_ENDPOINT"
        value = var.azure_openai_endpoint
      }
      env {
        name  = "AZURE_OPENAI_KEY"
        value = var.azure_openai_key
      }
    }
  }
  
  lifecycle {
    ignore_changes = [
      template[0].container[0].image
    ]
  }
}
