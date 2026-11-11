output "container_app_url" {
  description = "The public URL of the Collab Azure Container App"
  value       = "https://${azurerm_container_app.collab.latest_revision_fqdn}"
}

output "azure_ad_client_id" {
  description = "The Client ID of the generated Azure AD App Registration"
  value       = azuread_application.collab.application_id
}

output "acr_login_server" {
  description = "The Azure Container Registry login server"
  value       = azurerm_container_registry.acr.login_server
}
