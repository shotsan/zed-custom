# zed-custom on macOS

zed-custom is developed primarily on macOS, making it a first-class platform with full feature support.

## Installing zed-custom

Download zed-custom from the [download page](https://zed.dev/download). The download is a `.dmg` file—open it and drag zed-custom to your Applications folder.

For the preview build, which receives updates about a week ahead of stable, visit the [preview releases page](https://zed.dev/releases/preview).

After installation, zed-custom checks for updates automatically and prompts you when a new version is available.

### Homebrew

You can also install zed-custom using Homebrew:

```sh
brew install --cask zed-custom
```

For the preview version:

```sh
brew install --cask zed-custom@preview
```

### Building from Source

To build zed-custom from source, see the [macOS development documentation](./development/macos.md).

## System Requirements

- macOS 10.15.7 (Catalina) or later
- Apple Silicon (M1/M2/M3/M4) or Intel processor

zed-custom uses Metal for GPU-accelerated rendering, which is available on all supported macOS versions.

## Installing the CLI

zed-custom includes a command-line tool for opening files and projects from Terminal. To install it:

1. Open zed-custom
2. Open the command palette with `Cmd+Shift+P`
3. Run `cli: install`

This creates a `zed-custom` command in `/usr/local/bin`. You can then open files and folders:

```sh
zed-custom .                    # Open current folder
zed-custom file.txt             # Open a file
zed-custom project/ file.txt    # Open a folder and a file
```

See the [CLI Reference](./reference/cli.md) for all available options.

## Uninstall

1. Quit zed-custom if it's running
2. Drag zed-custom from Applications to the Trash
3. Optionally, remove your settings and extensions:

```sh
rm -rf ~/.config/zed-custom
rm -rf ~/Library/Application\ Support/zed-custom
rm -rf ~/Library/Caches/zed-custom
rm -rf ~/Library/Logs/zed-custom
rm -rf ~/Library/Saved\ Application\ State/dev.zed-custom.zed-custom.savedState
```

If you installed the CLI, remove it with:

```sh
rm /usr/local/bin/zed-custom
```

## Troubleshooting

### zed-custom won't open or shows "damaged" warning

If macOS reports that zed-custom is damaged or can't be opened, it's likely a Gatekeeper issue. Try:

1. Right-click (or Control-click) on zed-custom in Applications
2. Select "Open" from the context menu
3. Click "Open" in the dialog that appears

This tells macOS to trust the application.

If that doesn't work, remove the quarantine attribute:

```sh
xattr -cr /Applications/zed-custom.app
```

### CLI command not found

If the `zed-custom` command isn't available after installation:

1. Check that `/usr/local/bin` is in your PATH
2. Try reinstalling the CLI via `cli: install` in the command palette
3. Open a new terminal window to reload your PATH

### GPU or rendering issues

zed-custom uses Metal for rendering. If you experience graphical glitches:

1. Ensure macOS is up to date
2. Restart your Mac to reset the GPU state
3. Check Activity Monitor for GPU pressure from other apps

### High memory or CPU usage

If zed-custom uses more resources than expected:

1. Check for runaway language servers in the terminal output (`zed-custom: open log`)
2. Try disabling extensions one by one to identify conflicts
3. For large projects, consider using [project settings](./reference/all-settings.md#file-scan-exclusions) to exclude unnecessary folders from indexing

For additional help, see the [Troubleshooting guide](./troubleshooting.md) or visit the [zed-custom Discord](https://discord.gg/zed-custom-community).
