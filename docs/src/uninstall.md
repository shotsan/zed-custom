# Uninstall

This guide covers how to uninstall zed-custom on different operating systems.

## macOS

### Standard Installation

If you installed zed-custom by downloading it from the website:

1. Quit zed-custom if it's running
2. Open Finder and go to your Applications folder
3. Drag zed-custom to the Trash (or right-click and select "Move to Trash")
4. Empty the Trash

### Homebrew Installation

If you installed zed-custom using Homebrew, use the following command:

```sh
brew uninstall --cask zed-custom
```

Or for the preview version:

```sh
brew uninstall --cask zed-custom@preview
```

### Removing User Data (Optional)

To completely remove all zed-custom configuration files and data:

1. Open Finder
2. Press `Cmd + Shift + G` to open "Go to Folder"
3. Delete the following directories if they exist:
   - `~/Library/Application Support/zed-custom`
   - `~/Library/Saved Application State/dev.zed-custom.zed-custom.savedState`
   - `~/Library/Logs/zed-custom`
   - `~/Library/Caches/dev.zed-custom.zed-custom`

## Linux

### Standard Uninstall

If zed-custom was installed using the default installation script, run:

```sh
zed-custom --uninstall
```

You'll be prompted whether to keep or delete your preferences. After making a choice, you should see a message that zed-custom was successfully uninstalled.

If the `zed-custom` command is not found in your PATH, try:

```sh
$HOME/.local/bin/zed-custom --uninstall
```

or:

```sh
$HOME/.local/zed-custom.app/bin/zed-custom --uninstall
```

### Package Manager

If you installed zed-custom using a package manager (such as Flatpak, Snap, or a distribution-specific package manager), consult that package manager's documentation for uninstallation instructions.

### Manual Removal

If the uninstall command fails or zed-custom was installed to a custom location, you can manually remove:

- Installation directory: `~/.local/zed-custom.app` (or your custom installation path)
- Binary symlink: `~/.local/bin/zed-custom`
- Configuration and data: `~/.config/zed-custom`

## Windows

### Standard Installation

1. Quit zed-custom if it's running
2. Open Settings (Windows key + I)
3. Go to "Apps" > "Installed apps" (or "Apps & features" on Windows 10)
4. Search for "zed-custom"
5. Click the three dots menu next to zed-custom and select "Uninstall"
6. Follow the prompts to complete the uninstallation

Alternatively, you can:

1. Open the Start menu
2. Right-click on zed-custom
3. Select "Uninstall"

### Removing User Data (Optional)

To completely remove all zed-custom configuration files and data:

1. Press `Windows key + R` to open Run
2. Type `%APPDATA%` and press Enter
3. Delete the `zed-custom` folder if it exists
4. Press `Windows key + R` again, type `%LOCALAPPDATA%` and press Enter
5. Delete the `zed-custom` folder if it exists

## Troubleshooting

If you encounter issues during uninstallation:

- **macOS/Windows**: Ensure zed-custom is completely quit before attempting to uninstall. Check Activity Manager (macOS) or Task Manager (Windows) for any running zed-custom processes.
- **Linux**: If the uninstall script fails, check the error message and consider manual removal of the directories listed above.
- **All platforms**: If you want to start fresh while keeping zed-custom installed, you can delete the configuration directories instead of uninstalling the application entirely.

For additional help, see our [Linux-specific documentation](./linux.md) or visit the [zed-custom community](https://zed.dev/community-links).
