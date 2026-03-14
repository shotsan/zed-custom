# CLI Reference

zed-custom includes a command-line interface (CLI) for opening files and directories, integrating with other tools, and controlling zed-custom from scripts.

## Installation

**macOS:** Run the `cli: install` command from the command palette ({#kb command_palette::Toggle}) to install the `zed-custom` CLI to `/usr/local/bin/zed-custom`.

**Linux:** The CLI is included with zed-custom packages. The binary name may vary by distribution (commonly `zed-custom` or `zeditor`).

**Windows:** The CLI is included with zed-custom. Add zed-custom's installation directory to your PATH, or use the full path to `zed-custom.exe`.

## Usage

```sh
zed-custom [OPTIONS] [PATHS]...
```

## Opening Files and Directories

Open a file:

```sh
zed-custom myfile.txt
```

Open a directory as a workspace:

```sh
zed-custom ~/projects/myproject
```

Open multiple files or directories:

```sh
zed-custom file1.txt file2.txt ~/projects/myproject
```

Open a file at a specific line and column:

```sh
zed-custom myfile.txt:42        # Open at line 42
zed-custom myfile.txt:42:10     # Open at line 42, column 10
```

## Options

### `-w`, `--wait`

Wait for all opened files to be closed before the CLI exits. When opening a directory, waits until the window is closed.

This is useful for integrating zed-custom with tools that expect an editor to block until editing is complete (e.g., `git commit`):

```sh
export EDITOR="zed-custom --wait"
git commit  # Opens zed-custom and waits for you to close the commit message file
```

### `-n`, `--new`

Open paths in a new workspace window, even if the paths are already open in an existing window:

```sh
zed-custom -n ~/projects/myproject
```

### `-a`, `--add`

Add paths to the currently focused workspace instead of opening a new window:

```sh
zed-custom -a newfile.txt
```

### `-r`, `--reuse`

Reuse an existing window, replacing its current workspace with the new paths:

```sh
zed-custom -r ~/projects/different-project
```

### `--diff <OLD_PATH> <NEW_PATH>`

Open a diff view comparing two files. Can be specified multiple times:

```sh
zed-custom --diff file1.txt file2.txt
zed-custom --diff old.rs new.rs --diff old2.rs new2.rs
```

### `--foreground`

Run zed-custom in the foreground, keeping the terminal attached. Useful for debugging:

```sh
zed-custom --foreground
```

### `--user-data-dir <DIR>`

Use a custom directory for all user data (database, extensions, logs) instead of the default location:

```sh
zed-custom --user-data-dir ~/.zed-custom
```

Default locations:

- **macOS:** `~/Library/Application Support/zed-custom`
- **Linux:** `$XDG_DATA_HOME/zed-custom` (typically `~/.local/share/zed-custom`)
- **Windows:** `%LOCALAPPDATA%\zed-custom`

### `-v`, `--version`

Print zed-custom's version and exit:

```sh
zed-custom --version
```

### `--uninstall`

Uninstall zed-custom and remove all related files (macOS and Linux only):

```sh
zed-custom --uninstall
```

### `--zed-custom <PATH>`

Specify a custom path to the zed-custom application or binary:

```sh
zed-custom --zed-custom /path/to/zed-custom.app myfile.txt
```

## Reading from Standard Input

Read content from stdin by passing `-` as the path:

```sh
echo "Hello, World!" | zed-custom -
cat myfile.txt | zed-custom -
ps aux | zed-custom -
```

This creates a temporary file with the stdin content and opens it in zed-custom.

## URL Handling

The CLI can open `zed-custom://`, `http://`, and `https://` URLs:

```sh
zed-custom zed-custom://settings
zed-custom https://github.com/zed-industries/zed-custom
```

## Using zed-custom as Your Default Editor

Set zed-custom as your default editor for Git and other tools:

```sh
export EDITOR="zed-custom --wait"
export VISUAL="zed-custom --wait"
```

Add these lines to your shell configuration file (e.g., `~/.bashrc`, `~/.zshrc`).

## macOS: Switching Release Channels

On macOS, you can launch a specific release channel by passing the channel name as the first argument:

```sh
zed-custom --stable myfile.txt
zed-custom --preview myfile.txt
zed-custom --nightly myfile.txt
```

## WSL Integration (Windows)

On Windows, the CLI supports opening paths from WSL distributions. This is handled automatically when launching zed-custom from within WSL.

## Exit Codes

| Code | Meaning                           |
| ---- | --------------------------------- |
| `0`  | Success                           |
| `1`  | Error (details printed to stderr) |

When using `--wait`, the exit code reflects whether the files were saved before closing.
