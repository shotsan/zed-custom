# Original User Request

## Initial Request — 2026-07-16T13:28:30-07:00Z

# Teamwork Project Prompt — Draft

Implement an Azure AD (Entra ID) Enterprise OAuth flow in the Zed Collab backend, replacing the default GitHub OAuth logic. The team will build the necessary Axum endpoints (`/auth/login` and `/auth/callback`) and handle user persistence.

Integrity mode: development

## Requirements

### R1. Azure AD OAuth Flow
Implement the `/auth/login` and `/auth/callback` Axum endpoints in `crates/collab/src/api.rs` (or `crates/collab/src/auth.rs`). The endpoints must handle the standard OAuth 2.0 authorization code flow with Microsoft Entra ID (Azure AD) to fetch the user's email address.

### R2. User Persistence & Token Generation
Map the authenticated Azure AD email to the existing `users` table logic. Do not write any new database migrations. Generate the standard Zed native app access token (using the existing `create_access_token` flow) and redirect the user back to the application via the `zed://login` URL scheme.

## Acceptance Criteria

### OAuth & Persistence Tests
- [ ] A Rust test suite exists that mocks the Azure AD token and userinfo endpoints.
- [ ] The test programmatically executes the full `/auth/login` and `/auth/callback` flow.
- [ ] The test verifies that a new user record is created in the mocked database using their Azure AD email.
- [ ] The test verifies that the callback successfully returns an HTTP redirect to `zed://login` containing a valid encrypted access token.
