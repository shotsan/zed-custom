# Zed Custom Release Guide

This document explains how to trigger and manage releases for your custom fork of Zed.

## 🚀 How to Trigger a Release

The release process is fully automated via GitHub Actions. It is triggered whenever a new version tag is pushed to the repository.

### 1. Ensure your code is ready
Make sure all your local changes are committed and pushed to the `main` branch.

```bash
git add .
git commit -m "Your descriptive message"
git push origin main
```

### 2. Create and push a version tag
The workflow looks for tags starting with `v` (e.g., `v0.1.0`, `v1.2.3`).

```bash
# Create a new local tag
git tag v0.1.1

# Push the tag to GitHub
git push origin v0.1.1
```

### 3. Monitor the Build
Once the tag is pushed, go to the **Actions** tab in your GitHub repository. You will see a workflow named **"Release Binaries"** running.

- **Duration**: Approximately 20-30 minutes.
- **Platforms**: Builds for macOS (Intel & Apple Silicon) and Linux (x86_64).
- **Note**: Windows builds are currently disabled to ensure release stability.

---

## 📦 What happens during the release?

The GitHub Action performs the following steps:
1. **Compilation**: Builds the `zed`, `cli`, and `remote_server` binaries for all supported platforms.
2. **Bundling**: Packages the binaries into user-ready formats (`.dmg` for Mac, `.tar.gz` for Linux).
3. **Drafting Release**: Creates a new entry in your repository's "Releases" section.
4. **Notes Generation**: Automatically generates release notes based on the commit history since the last tag.
5. **Asset Upload**: Attaches all compiled binaries to the release.

---

## 🛠 Troubleshooting

### "The build failed on Linux"
If the Linux build fails with linker errors related to `__isoc23`, ensure the workflow is running on `ubuntu-22.04` (pinned in `.github/workflows/release-binaries.yml`).

### "I need to fix a tag I just pushed"
If you pushed a tag but realized something was wrong, you must delete the tag on both local and remote before re-pushing:

```bash
# Delete locally
git tag -d v0.1.1

# Delete on GitHub
git push --delete origin v0.1.1

# ... make your fixes and commit them ...

# Re-tag and push again
git tag v0.1.1
git push origin v0.1.1
```

### "Where are the binaries?"
Once the Action is completed (Green checkmark), the binaries are available at:
`https://github.com/shotsan/zed-custom/releases`
