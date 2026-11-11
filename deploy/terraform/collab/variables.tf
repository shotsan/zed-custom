variable "db_admin_user" {
  description = "Administrator login for PostgreSQL"
  type        = string
  default     = "zedadmin"
}

variable "azure_openai_endpoint" {
  type        = string
  description = "Azure OpenAI Endpoint URL"
  default     = "https://example.openai.azure.com/openai/deployments/gpt-4/chat/completions?api-version=2023-05-15"
}

variable "azure_openai_key" {
  type        = string
  description = "Azure OpenAI API Key"
  sensitive   = true
  default     = "placeholder-key"
}
