# E2E Test Infra: Zed Azure AD Enterprise Auth

## Test Philosophy
- Opaque-box, requirement-driven. No dependency on implementation design.
- Methodology: Category-Partition + BVA + Pairwise + Workload Testing.

## Feature Inventory
| # | Feature | Source (requirement) | Tier 1 | Tier 2 | Tier 3 |
|---|---------|---------------------|:------:|:------:|:------:|
| 1 | /auth/login Redirect | ORIGINAL_REQUEST R1 | 5      | 5      | ✓      |
| 2 | /auth/callback Exchange & Persistence | ORIGINAL_REQUEST R2 | 5      | 5      | ✓      |
| 3 | Access Token Generation & zed://login | ORIGINAL_REQUEST R2 | 5      | 5      | ✓      |

## Test Architecture
- Test runner: `cargo test -p collab --test azure_ad_e2e_tests` or similar existing test harness.
- Test case format: Rust tests using Axum testing utilities (e.g. `axum::test_helpers` or `reqwest` if running a test server) and a mocked Azure AD token/userinfo endpoint.
- Directory layout: `crates/collab/tests/azure_ad_e2e_tests.rs` or integrated into `crates/collab/src/api/auth_tests.rs`.

## Real-World Application Scenarios (Tier 4)
| # | Scenario | Features Exercised | Complexity |
|---|----------|--------------------|------------|
| 1 | Complete Auth Flow | F1, F2, F3 | High |
| 2 | Concurrent Logins | F1, F2, F3 | High |
| 3 | Re-login of Existing User | F1, F2, F3 | Medium |
| 4 | Token Refresh / Expiry | F3 | Medium |
| 5 | Missing Config Graceful Failure | F1 | Low |

## Coverage Thresholds
- Tier 1: ≥5 per feature
- Tier 2: ≥5 per feature (where boundaries exist)
- Tier 3: pairwise coverage of major feature interactions
- Tier 4: ≥5 realistic application scenarios
