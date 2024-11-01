#!/bin/bash

# Determine platform-specific variables
case "$(uname -s)" in
    Darwin*)
        PLATFORM="apple-darwin"
        INSTALL_DIR="/usr/local/bin"
        EXE_NAME="stitch-sync"
        ;;
    Linux*)
        PLATFORM="unknown-linux-gnu"
        INSTALL_DIR="/usr/local/bin"
        EXE_NAME="stitch-sync"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="pc-windows-msvc"
        INSTALL_DIR="/c/Windows/System32"
        EXE_NAME="stitch-sync.exe"
        ;;
    *)
        echo "Unsupported platform: $(uname -s)"
        exit 1
        ;;
esac

# Get latest release info from GitHub API
echo "Fetching latest release information..."
RELEASE_INFO=$(curl -s "https://api.github.com/repos/osteele/stitchsync/releases/latest")
if [ $? -ne 0 ]; then
    echo "Failed to fetch release information"
    exit 1
fi

# Extract version tag from release info
RELEASE_VERSION=$(echo "$RELEASE_INFO" | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4)
if [ -z "$RELEASE_VERSION" ]; then
    echo "Failed to determine latest version"
    exit 1
fi

echo "Latest version: $RELEASE_VERSION"
ASSET_NAME="stitch-sync-x86_64-${PLATFORM}.tar.gz"
DOWNLOAD_URL="https://github.com/osteele/stitchsync/releases/download/${RELEASE_VERSION}/${ASSET_NAME}"

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR" || exit 1

echo "Downloading StitchSync ${RELEASE_VERSION} for ${PLATFORM}..."
if ! curl -L -o "$ASSET_NAME" "$DOWNLOAD_URL"; then
    echo "Failed to download release"
    rm -rf "$TMP_DIR"
    exit 1
fi

echo "Extracting archive..."
if ! tar xzf "$ASSET_NAME"; then
    echo "Failed to extract archive"
    rm -rf "$TMP_DIR"
    exit 1
fi

echo "Installing to ${INSTALL_DIR}..."
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating installation directory..."
    sudo mkdir -p "$INSTALL_DIR"
fi

if ! mv "$EXE_NAME" "$INSTALL_DIR/"; then
    echo "Failed to install executable. Do you have the necessary permissions?"
    rm -rf "$TMP_DIR"
    exit 1
fi

# Clean up
cd - > /dev/null
rm -rf "$TMP_DIR"

echo "Successfully installed StitchSync ${RELEASE_VERSION} to ${INSTALL_DIR}/${EXE_NAME}"
echo "Run 'stitch-sync --help' to get started"
