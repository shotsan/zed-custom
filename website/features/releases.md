# 📦 Automated Cross-Platform Releases

Maintaining parity with upstream Zed while distributing custom builds is hard. The upstream repository relies on massive internal Namespace runners to bundle their binary assets, creating friction for individual contributors trying to release their forks.

This repository implements a completely automated, public GitHub Actions CI/CD release pipeline using standard runners (`macos-13`, `macos-14`, `ubuntu-22.04`).

## Features

1. **Native Apple Notarization**: The workflow handles the 3-step code signing and external Apple Notarization Server API flow natively. It strips the binaries, extracts debug symbols (`sentry-cli`), signs the app bundle with the `Developer ID Application` certificate, and waits for successful external notarization before stapling. 
2. **Parallel Architecture Matrix**: The 150-minute Apple Notarization limits are bypassed by breaking builds into isolated parallel matrices (`x86_64` and `aarch64` building concurrently on separate VMs).
3. **Linux Bundles out-of-the-box**: Using the exact upstream compilation targets, the Linux pipelines output `tar.gz` and remote server `.gz` architectures natively via `ubuntu-22.04` and `mold`.

### Triggering a Release
Every time you push an annotated git tag prefixed with `v` (e.g. `v0.2.1-pre11`), the pipeline boots up, builds the entire Rust application across all OS architectures, and uploads the artifacts as pristine GitHub Assets on the Releases page instantly.
