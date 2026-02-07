#!/bin/sh
# mcp-scanner installer
# Usage: curl -fsSL https://raw.githubusercontent.com/oabraham1/mcp-scanner/main/install.sh | sh

set -e

REPO="oabraham1/mcp-scanner"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            case "$ARCH" in
                x86_64) PLATFORM="x86_64-unknown-linux-gnu" ;;
                aarch64) PLATFORM="aarch64-unknown-linux-gnu" ;;
                *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
            esac
            ;;
        Darwin*)
            case "$ARCH" in
                x86_64) PLATFORM="x86_64-apple-darwin" ;;
                arm64) PLATFORM="aarch64-apple-darwin" ;;
                *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            PLATFORM="x86_64-pc-windows-msvc"
            ;;
        *)
            echo "Unsupported OS: $OS"
            exit 1
            ;;
    esac
}

# Get latest release tag
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
        grep '"tag_name":' |
        sed -E 's/.*"([^"]+)".*/\1/'
}

main() {
    echo "Installing mcp-scanner..."

    detect_platform
    VERSION=$(get_latest_version)

    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine latest version"
        exit 1
    fi

    echo "  Platform: $PLATFORM"
    echo "  Version: $VERSION"

    # Download URL
    if [ "$PLATFORM" = "x86_64-pc-windows-msvc" ]; then
        ARCHIVE="mcp-scanner-${PLATFORM}.zip"
    else
        ARCHIVE="mcp-scanner-${PLATFORM}.tar.gz"
    fi

    URL="https://github.com/$REPO/releases/download/$VERSION/$ARCHIVE"

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    echo "  Downloading from $URL..."
    curl -fsSL "$URL" -o "$TMP_DIR/$ARCHIVE"

    # Extract
    echo "  Extracting..."
    cd "$TMP_DIR"
    if [ "$PLATFORM" = "x86_64-pc-windows-msvc" ]; then
        unzip -q "$ARCHIVE"
    else
        tar xzf "$ARCHIVE"
    fi

    # Install
    echo "  Installing to $INSTALL_DIR..."
    if [ -w "$INSTALL_DIR" ]; then
        mv mcp-scanner "$INSTALL_DIR/"
    else
        sudo mv mcp-scanner "$INSTALL_DIR/"
    fi

    chmod +x "$INSTALL_DIR/mcp-scanner"

    echo ""
    echo "mcp-scanner $VERSION installed successfully!"
    echo ""
    echo "Run 'mcp-scanner --help' to get started."
}

main
