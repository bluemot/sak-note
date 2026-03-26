#!/bin/bash
# SAK Editor - Environment Setup Script for Linux/macOS
# This script installs all dependencies needed to build and run SAK Editor

# Show help
if [[ "$1" == "--help" || "$1" == "-h" || "$1" == "help" ]]; then
    echo ""
    echo "Usage: ./install.sh [OPTIONS]"
    echo ""
    echo "Installs all dependencies for SAK Editor:"
    echo "  - Node.js (v20+)"
    echo "  - Rust & Cargo"
    echo "  - Tauri CLI"
    echo "  - System dependencies"
    echo "  - Project npm packages"
    echo ""
    echo "Options:"
    echo "  -h, --help, help    Show this help message"
    echo ""
    echo "After install, run:"
    echo "  ./build.sh          Build release version"
    echo "  npm run dev         Start dev server"
    echo "  npm test            Run tests"
    echo ""
    exit 0
fi

set -e

echo "=========================================="
echo "SAK Editor - Environment Setup"
echo "=========================================="
echo ""
echo "Run './install.sh help' for usage info"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Detect OS
OS="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
fi

echo "Detected OS: $OS"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Install Node.js and npm
install_node() {
    echo "Checking Node.js..."
    if command_exists node; then
        NODE_VERSION=$(node --version)
        echo -e "${GREEN}[OK]${NC} Node.js already installed: $NODE_VERSION"
    else
        echo -e "${YELLOW}Installing Node.js...${NC}"
        
        # Ensure curl is available for nodesource setup
        if ! command_exists curl; then
            echo -e "${RED}[ERR]${NC} curl is required but not found. Please install curl first."
            exit 1
        fi

        if [ "$OS" == "linux" ]; then
            if command_exists apt-get; then
                curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
                sudo apt-get install -f -y nodejs
            elif command_exists yum; then
                curl -fsSL https://rpm.nodesource.com/setup_20.x | sudo bash -
                sudo yum install -y nodejs
            else
                echo -e "${RED}[ERR]${NC} Please install Node.js manually from https://nodejs.org/"
                exit 1
            fi
        elif [ "$OS" == "macos" ]; then
            if command_exists brew; then
                brew install node
            else
                echo -e "${RED}[ERR]${NC} Please install Homebrew first: https://brew.sh/"
                exit 1
            fi
        fi
        echo -e "${GREEN}[OK]${NC} Node.js installed"
    fi
    echo ""
}

# Install Rust and Cargo
install_rust() {
    echo "Checking Rust..."
    if command_exists cargo; then
        RUST_VERSION=$(rustc --version)
        echo -e "${GREEN}[OK]${NC} Rust already installed: $RUST_VERSION"
    else
        echo -e "${YELLOW}Installing Rust...${NC}"
        
        if ! command_exists curl; then
            echo -e "${RED}[ERR]${NC} curl is required but not found. Please install curl first."
            exit 1
        fi

        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo -e "${GREEN}[OK]${NC} Rust installed"
    fi
    echo ""
}

# Install system dependencies for Tauri with smart detection
install_tauri_deps() {
    echo "Checking Tauri system dependencies..."
    if [ "$OS" == "linux" ]; then
        echo "Installing Linux dependencies for Tauri..."
        if command_exists apt-get; then
            sudo apt-get update

            # Define base packages that rarely change names
            local APT_PACKAGES="libgtk-3-dev librsvg2-dev patchelf"

            # Smart detection for webkit2gtk
            if apt-cache show libwebkit2gtk-4.1-dev >/dev/null 2>&1; then
                echo "Found webkit2gtk-4.1, adding to install list..."
                APT_PACKAGES="$APT_PACKAGES libwebkit2gtk-4.1-dev"
            elif apt-cache show libwebkit2gtk-4.0-dev >/dev/null 2>&1; then
                echo "Fallback to webkit2gtk-4.0..."
                APT_PACKAGES="$APT_PACKAGES libwebkit2gtk-4.0-dev"
            fi

            # Smart detection for appindicator
            if apt-cache show libayatana-appindicator3-dev >/dev/null 2>&1; then
                echo "Found modern ayatana-appindicator, adding to install list..."
                APT_PACKAGES="$APT_PACKAGES libayatana-appindicator3-dev"
            elif apt-cache show libappindicator3-dev >/dev/null 2>&1; then
                echo "Fallback to legacy appindicator..."
                APT_PACKAGES="$APT_PACKAGES libappindicator3-dev"
            fi

            echo "Final APT packages to install: $APT_PACKAGES"
            sudo apt-get install -f -y $APT_PACKAGES

        elif command_exists yum; then
            sudo yum install -y gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel
        elif command_exists pacman; then
            sudo pacman -S --needed --noconfirm gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg
        fi
    elif [ "$OS" == "macos" ]; then
        echo -e "${GREEN}[OK]${NC} macOS dependencies usually pre-installed"
    fi
    echo -e "${GREEN}[OK]${NC} Tauri dependencies installed"
    echo ""
}

# Install project dependencies
install_project_deps() {
    echo "Installing project dependencies..."
    
    # Get script directory safely
    local SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR"
    
    # Install root dependencies
    echo "Installing root npm dependencies..."
    npm install
    
    # Install frontend dependencies
    echo "Installing frontend npm dependencies..."
    cd src-frontend
    npm install
    cd ..
    
    echo -e "${GREEN}[OK]${NC} Project dependencies installed"
    echo ""
}

# Install Tauri CLI
install_tauri_cli() {
    echo "Installing Tauri CLI..."
    source "$HOME/.cargo/env" 2>/dev/null || true
    if ! command_exists cargo-tauri; then
        echo "cargo-tauri not found, installing via cargo..."
        cargo install tauri-cli
    else
        echo "cargo-tauri is already installed."
    fi
    echo -e "${GREEN}[OK]${NC} Tauri CLI ready"
    echo ""
}

# Main installation process
main() {
    install_node
    install_rust
    install_tauri_deps
    install_project_deps
    install_tauri_cli
    
    echo "=========================================="
    echo -e "${GREEN}Setup complete!${NC}"
    echo "=========================================="
    echo ""
    echo "You can now run:"
    echo "  npm run dev       - Start development server"
    echo "  npm run tauri-dev - Start Tauri development"
    echo "  npm run build     - Build release version"
    echo "  npm test          - Run all tests"
    echo ""
}

main