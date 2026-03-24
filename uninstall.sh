#!/bin/bash
set -e

# greq uninstall script
# Usage: curl -sSL https://raw.githubusercontent.com/KlausSchaefers/greq/main/uninstall.sh | bash

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

# Detect OS to set correct binary name
detect_os() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    
    case "$os" in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="macos"
            ;;
        cygwin*|mingw*|msys*)
            OS="windows"
            BINARY_NAME="greq.exe"
            ;;
        *)
            warn "Unknown operating system: $os"
            warn "Assuming Unix-like system, will try to remove 'greq'"
            ;;
    esac
    
    log "Detected OS: $OS"
}

# Check if greq is installed
check_installation() {
    local binary_path="$INSTALL_DIR/$BINARY_NAME"
    
    if [ -f "$binary_path" ]; then
        log "Found greq installation at: $binary_path"
        return 0
    fi
    
    # Check if it's in PATH but not in standard location
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local found_path=$(which "$BINARY_NAME")
        warn "greq found at non-standard location: $found_path"
        warn "This script only removes greq from $INSTALL_DIR"
        warn "To remove from $found_path, run:"
        warn "  sudo rm '$found_path'"
        return 1
    fi
    
    error "greq is not installed at $binary_path"
}

# Remove the binary
remove_binary() {
    local binary_path="$INSTALL_DIR/$BINARY_NAME"
    
    log "Removing greq from $binary_path..."
    
    # Try to remove with sudo first, then without if that fails
    if sudo rm "$binary_path" 2>/dev/null; then
        success "Binary removed successfully with sudo"
    elif rm "$binary_path" 2>/dev/null; then
        success "Binary removed successfully"
    else
        error "Failed to remove $binary_path. Please check permissions or run with sudo."
    fi
}

# Verify removal
verify_removal() {
    local binary_path="$INSTALL_DIR/$BINARY_NAME"
    
    if [ ! -f "$binary_path" ]; then
        success "greq has been completely removed from $binary_path"
        
        # Check if it's still in PATH from another location
        if command -v "$BINARY_NAME" >/dev/null 2>&1; then
            local remaining_path=$(which "$BINARY_NAME")
            warn "Note: greq is still available from: $remaining_path"
            warn "This may be a different installation or shell alias"
        fi
    else
        error "Failed to remove greq. File still exists at $binary_path"
    fi
}

# Clean up shell configurations (optional)
cleanup_shell_config() {
    log "Checking for greq-related shell configurations..."
    
    local config_files=(
        "$HOME/.bashrc"
        "$HOME/.zshrc"
        "$HOME/.bash_profile"
        "$HOME/.profile"
    )
    
    local found_configs=0
    
    for config_file in "${config_files[@]}"; do
        if [ -f "$config_file" ] && grep -q "greq" "$config_file"; then
            warn "Found greq references in $config_file"
            warn "You may want to manually review and remove them"
            found_configs=1
        fi
    done
    
    if [ $found_configs -eq 0 ]; then
        log "No greq references found in shell configuration files"
    fi
}

# Show uninstall summary
show_summary() {
    echo ""
    echo "🗑️  greq Uninstall Complete"
    echo "=========================="
    echo ""
    success "greq has been successfully removed from your system"
    echo ""
    echo "What was removed:"
    echo "  • Binary: $INSTALL_DIR/$BINARY_NAME"
    if [ "$OS" = "macos" ]; then
        echo "  • macOS quarantine attributes (automatically cleaned)"
    fi
    echo ""
    echo "What was NOT removed (intentionally):"
    echo "  • Your files and data (greq doesn't store any)"
    echo "  • Shell configuration files (may contain PATH modifications)"
    echo ""
    echo "If you want to reinstall greq later:"
    echo "  curl -sSL https://raw.githubusercontent.com/KlausSchaefers/greq/main/install.sh | bash"
    echo ""
}

# Confirmation prompt
confirm_uninstall() {
    echo "🗑️  greq Uninstaller"
    echo "==================="
    echo ""
    echo "This will remove greq from: $INSTALL_DIR/$BINARY_NAME"
    echo ""
    
    # Check if running in non-interactive mode (CI/automated)
    if [ ! -t 0 ]; then
        log "Running in non-interactive mode, proceeding with uninstall..."
        return 0
    fi
    
    read -p "Are you sure you want to uninstall greq? [y/N]: " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "Uninstall cancelled by user"
        exit 0
    fi
}

# Main uninstall process
main() {
    detect_os
    confirm_uninstall
    check_installation
    remove_binary
    verify_removal
    cleanup_shell_config
    show_summary
}

# Run main function
main "$@"