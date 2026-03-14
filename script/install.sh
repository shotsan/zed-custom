#!/usr/bin/env sh
set -eu

# Downloads a tarball from https://zed.dev/releases and unpacks it
# into ~/.local/. If you'd prefer to do this manually, instructions are at
# https://zed.dev/docs/linux.

main() {
    platform="$(uname -s)"
    arch="$(uname -m)"
    channel="${ZED_CHANNEL:-stable}"
    ZED_VERSION="${ZED_VERSION:-latest}"
    # Use TMPDIR if available (for environments with non-standard temp directories)
    if [ -n "${TMPDIR:-}" ] && [ -d "${TMPDIR}" ]; then
        temp="$(mktemp -d "$TMPDIR/zed-custom-XXXXXX")"
    else
        temp="$(mktemp -d "/tmp/zed-custom-XXXXXX")"
    fi

    if [ "$platform" = "Darwin" ]; then
        platform="macos"
    elif [ "$platform" = "Linux" ]; then
        platform="linux"
    else
        echo "Unsupported platform $platform"
        exit 1
    fi

    case "$platform-$arch" in
        macos-arm64* | linux-arm64* | linux-armhf | linux-aarch64)
            arch="aarch64"
            ;;
        macos-x86* | linux-x86* | linux-i686*)
            arch="x86_64"
            ;;
        *)
            echo "Unsupported platform or architecture"
            exit 1
            ;;
    esac

    if command -v curl >/dev/null 2>&1; then
        curl () {
            command curl -fL "$@"
        }
    elif command -v wget >/dev/null 2>&1; then
        curl () {
            wget -O- "$@"
        }
    else
        echo "Could not find 'curl' or 'wget' in your path"
        exit 1
    fi

    "$platform" "$@"

    if [ "$(command -v zed-custom)" = "$HOME/.local/bin/zed-custom" ]; then
        echo "zed-custom has been installed. Run with 'zed-custom'"
    else
        echo "To run zed-custom from your terminal, you must add ~/.local/bin to your PATH"
        echo "Run:"

        case "$SHELL" in
            *zsh)
                echo "   echo 'export PATH=\$HOME/.local/bin:\$PATH' >> ~/.zshrc"
                echo "   source ~/.zshrc"
                ;;
            *fish)
                echo "   fish_add_path -U $HOME/.local/bin"
                ;;
            *)
                echo "   echo 'export PATH=\$HOME/.local/bin:\$PATH' >> ~/.bashrc"
                echo "   source ~/.bashrc"
                ;;
        esac

        echo "To run zed-custom now, '~/.local/bin/zed-custom'"
    fi
}

linux() {
    if [ -n "${ZED_BUNDLE_PATH:-}" ]; then
        cp "$ZED_BUNDLE_PATH" "$temp/zed-custom-linux-$arch.tar.gz"
    else
        echo "Downloading zed-custom version: $ZED_VERSION"
        curl "https://cloud.zed.dev/releases/$channel/$ZED_VERSION/download?asset=zed-custom&arch=$arch&os=linux&source=install.sh" > "$temp/zed-custom-linux-$arch.tar.gz"
    fi

    suffix=""
    if [ "$channel" != "stable" ]; then
        suffix="-$channel"
    fi

    appid=""
    case "$channel" in
      stable)
        appid="dev.zed-custom.zed-custom"
        ;;
      nightly)
        appid="dev.zed-custom.zed-custom-Nightly"
        ;;
      preview)
        appid="dev.zed-custom.zed-custom-Preview"
        ;;
      dev)
        appid="dev.zed-custom.zed-custom-Dev"
        ;;
      *)
        echo "Unknown release channel: ${channel}. Using stable app ID."
        appid="dev.zed-custom.zed-custom"
        ;;
    esac

    # Unpack
    rm -rf "$HOME/.local/zed-custom$suffix.app"
    mkdir -p "$HOME/.local/zed-custom$suffix.app"
    tar -xzf "$temp/zed-custom-linux-$arch.tar.gz" -C "$HOME/.local/"

    # Setup ~/.local directories
    mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"

    # Link the binary
    if [ -f "$HOME/.local/zed-custom$suffix.app/bin/zed-custom" ]; then
        ln -sf "$HOME/.local/zed-custom$suffix.app/bin/zed-custom" "$HOME/.local/bin/zed-custom"
    else
        # support for versions before 0.139.x.
        ln -sf "$HOME/.local/zed-custom$suffix.app/bin/cli" "$HOME/.local/bin/zed-custom"
    fi

    # Copy .desktop file
    desktop_file_path="$HOME/.local/share/applications/${appid}.desktop"
    cp "$HOME/.local/zed-custom$suffix.app/share/applications/zed-custom$suffix.desktop" "${desktop_file_path}"
    sed -i "s|Icon=zed-custom|Icon=$HOME/.local/zed-custom$suffix.app/share/icons/hicolor/512x512/apps/zed-custom.png|g" "${desktop_file_path}"
    sed -i "s|Exec=zed-custom|Exec=$HOME/.local/zed-custom$suffix.app/bin/zed-custom|g" "${desktop_file_path}"
}

macos() {
    echo "Downloading zed-custom version: $ZED_VERSION"
    curl "https://cloud.zed.dev/releases/$channel/$ZED_VERSION/download?asset=zed-custom&os=macos&arch=$arch&source=install.sh" > "$temp/zed-custom-$arch.dmg"
    hdiutil attach -quiet "$temp/zed-custom-$arch.dmg" -mountpoint "$temp/mount"
    app="$(cd "$temp/mount/"; echo *.app)"
    echo "Installing $app"
    if [ -d "/Applications/$app" ]; then
        echo "Removing existing $app"
        rm -rf "/Applications/$app"
    fi
    ditto "$temp/mount/$app" "/Applications/$app"
    hdiutil detach -quiet "$temp/mount"

    mkdir -p "$HOME/.local/bin"
    # Link the binary
    ln -sf "/Applications/$app/Contents/MacOS/cli" "$HOME/.local/bin/zed-custom"
}

main "$@"
