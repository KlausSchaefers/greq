#!/bin/bash
set -e

# greq installer script
# Usage: curl -sSL https://raw.githubusercontent.com/KlausSchaefers/greq/main/install.sh | bash

REPO="KlausSchaefers/greq"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="greq"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case "$os" in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="macos"
            ;;
        cygwin*|mingw*|msys*)
            OS="windows"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac
    
    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="arm64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac
    
    # Determine the asset name based on platform
    case "$OS-$ARCH" in
        linux-x86_64)
            ASSET_NAME="greq"
            PLATFORM_NAME="Linux x86_64"
            ;;
        linux-arm64)
            ASSET_NAME="greq"
            PLATFORM_NAME="Linux ARM64"
            ;;
        macos-x86_64)
            ASSET_NAME="greq"
            PLATFORM_NAME="macOS Intel"
            ;;
        macos-arm64)
            ASSET_NAME="greq"
            PLATFORM_NAME="macOS ARM64"
            ;;
        windows-x86_64)
            ASSET_NAME="greq.exe"
            PLATFORM_NAME="Windows x86_64"
            BINARY_NAME="greq.exe"
            ;;
        *)
            error "No binary available for $OS-$ARCH"
            ;;
    esac
    
    log "Detected platform: $PLATFORM_NAME"
}

# Get the latest release download URL
get_download_url() {
    log "Fetching latest release information..."
    
    if command -v curl >/dev/null 2>&1; then
        local releases_url="https://api.github.com/repos/$REPO/releases/latest"
        local release_data=$(curl -s "$releases_url")
        
        if [ $? -ne 0 ] || [ -z "$release_data" ]; then
            error "Failed to fetch release information from GitHub"
        fi
        
        # Extract download URL - look for the asset with matching name
        DOWNLOAD_URL=$(echo "$release_data" | grep -o "https://github.com/$REPO/releases/download/[^\"]*/$ASSET_NAME" | head -1)
        
        if [ -z "$DOWNLOAD_URL" ]; then
            error "Could not find download URL for $ASSET_NAME in latest release"
        fi
        
        # Extract version tag
        VERSION=$(echo "$release_data" | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
        
        log "Found greq version: $VERSION"
        log "Download URL: $DOWNLOAD_URL"
    else
        error "curl is required but not installed"
    fi
}

# Download the binary
download_binary() {
    local temp_dir=$(mktemp -d)
    local temp_file="$temp_dir/$BINARY_NAME"
    
    log "Downloading $BINARY_NAME..."
    
    if ! curl -sL "$DOWNLOAD_URL" -o "$temp_file"; then
        error "Failed to download binary"
    fi
    
    if [ ! -f "$temp_file" ]; then
        error "Downloaded file not found"
    fi
    
    # Verify it's not empty
    if [ ! -s "$temp_file" ]; then
        error "Downloaded file is empty"
    fi
    
    TEMP_BINARY="$temp_file"
    success "Binary downloaded successfully"
}

# Install the binary
install_binary() {
    log "Installing $BINARY_NAME to $INSTALL_DIR..."
    
    # Check if install directory exists, create if it doesn't
    if [ ! -d "$INSTALL_DIR" ]; then
        warn "$INSTALL_DIR does not exist, attempting to create..."
        if ! sudo mkdir -p "$INSTALL_DIR" 2>/dev/null; then
            error "Cannot create $INSTALL_DIR. Please run with sudo or choose a different install directory."
        fi
    fi
    
    # Install binary
    if ! sudo cp "$TEMP_BINARY" "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null; then
        # Try without sudo for user directories
        if ! cp "$TEMP_BINARY" "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null; then
            error "Failed to install binary. Please check permissions or run with sudo."
        fi
    fi
    
    # Make executable
    if ! sudo chmod +x "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null; then
        chmod +x "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null || error "Failed to make binary executable"
    fi
    
    # Clean up temp file
    rm -f "$TEMP_BINARY"
    
    success "Binary installed to $INSTALL_DIR/$BINARY_NAME"
}

# Handle macOS security
handle_macos_security() {
    if [ "$OS" = "macos" ]; then
        log "Removing macOS quarantine attribute..."
        if sudo xattr -d com.apple.quarantine "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null; then
            success "Quarantine attribute removed"
        else
            warn "Could not remove quarantine attribute. You may need to run:"
            warn "  sudo xattr -d com.apple.quarantine $INSTALL_DIR/$BINARY_NAME"
        fi
    fi
}

# Check if binary is in PATH
check_path() {
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        success "$BINARY_NAME is now available in your PATH"
        log "Installed version: $(${BINARY_NAME} --version 2>/dev/null || echo 'unknown')"
    else
        warn "$INSTALL_DIR is not in your PATH"
        warn "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        warn "  export PATH=\"\$PATH:$INSTALL_DIR\""
        warn ""
        warn "Or run directly: $INSTALL_DIR/$BINARY_NAME"
    fi
}

# Test installation
test_installation() {
    if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
        success "Installation completed successfully!"
        echo ""
        echo "Try it out:"
        echo "  $BINARY_NAME --help"
        echo "  $BINARY_NAME 'search term' /path/to/search"
        echo ""
        if [ "$OS" = "macos" ]; then
            echo "Note: If you see a security warning, run:"
            echo "  sudo xattr -d com.apple.quarantine $INSTALL_DIR/$BINARY_NAME"
            echo ""
        fi
    else
        error "Installation verification failed"
    fi
}

# Main installation process
main() {
    echo "🔍 greq installer"
    echo "=================="
    echo ""
    
    detect_platform
    get_download_url
    download_binary
    install_binary
    handle_macos_security
    check_path
    test_installation
}

# Run main function
main "$@"