# Project: Zed Azure AD Enterprise Auth

## Architecture
- `crates/collab/src/api.rs`: Axum router registration for OAuth endpoints
- `crates/collab/src/api/azure_ad.rs` (or within `auth.rs`/`api.rs` directly): Logic for `/auth/login` and `/auth/callback`
- `crates/collab/src/auth.rs`: Authentication and token generation utilities
- External Identity Provider: Microsoft Entra ID (Azure AD)

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Azure AD Endpoints & Persistence | Implement `/auth/login` redirect and `/auth/callback` token exchange. Map Azure AD email to Zed users table, generate standard Zed native app access token, redirect to `zed://login` | none | PLANNED |
| Final | E2E Test Suite | Pass 100% of opaque-box E2E tests for Azure AD integration | 1 | PLANNED |

## Interface Contracts
### `axum` ↔ `Azure AD`
- `GET /auth/login` -> 302 Redirect to `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize` with standard parameters.
- `GET /auth/callback?code=...&state=...` -> Exchanges code for token at `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token`, fetches user profile, extracts email, creates/finds user in `users` table, creates access token, encrypts it, and returns 302 Redirect to `zed://login?access_token={encrypted_token}`

## Code Layout
- Existing conventions apply.
- Use `crates/collab/src/api.rs` for defining the new routes.
- Implement the actual handlers in `crates/collab/src/api/azure_ad.rs` (if creating a new module) or `crates/collab/src/auth.rs`.
