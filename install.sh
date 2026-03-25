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
        echo -e "${GREEN}✓${NC} Node.js already installed: $NODE_VERSION"
    else
        echo -e "${YELLOW}Installing Node.js...${NC}"
        if [ "$OS" == "linux" ]; then
            # Install Node.js using nvm or apt
            if command_exists apt; then
                curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
                sudo apt-get install -y nodejs
            elif command_exists yum; then
                curl -fsSL https://rpm.nodesource.com/setup_20.x | sudo bash -
                sudo yum install -y nodejs
            else
                echo -e "${RED}✗${NC} Please install Node.js manually from https://nodejs.org/"
                exit 1
            fi
        elif [ "$OS" == "macos" ]; then
            if command_exists brew; then
                brew install node
            else
                echo -e "${RED}✗${NC} Please install Homebrew first: https://brew.sh/"
                exit 1
            fi
        fi
        echo -e "${GREEN}✓${NC} Node.js installed"
    fi
    echo ""
}

# Install Rust and Cargo
install_rust() {
    echo "Checking Rust..."
    if command_exists cargo; then
        RUST_VERSION=$(rustc --version)
        echo -e "${GREEN}✓${NC} Rust already installed: $RUST_VERSION"
    else
        echo -e "${YELLOW}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo -e "${GREEN}✓${NC} Rust installed"
    fi
    echo ""
}

# Install system dependencies for Tauri
install_tauri_deps() {
    echo "Checking Tauri system dependencies..."
    if [ "$OS" == "linux" ]; then
        echo "Installing Linux dependencies for Tauri..."
        if command_exists apt; then
            sudo apt-get update
            sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
        elif command_exists yum; then
            sudo yum install -y gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel
        elif command_exists pacman; then
            sudo pacman -S --needed --noconfirm gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg
        fi
    elif [ "$OS" == "macos" ]; then
        echo -e "${GREEN}✓${NC} macOS dependencies usually pre-installed"
    fi
    echo -e "${GREEN}✓${NC} Tauri dependencies installed"
    echo ""
}

# Install project dependencies
install_project_deps() {
    echo "Installing project dependencies..."
    
    # Get script directory
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR"
    
    # Install root dependencies
    echo "Installing root npm dependencies..."
    npm install
    
    # Install frontend dependencies
    echo "Installing frontend npm dependencies..."
    cd src-frontend
    npm install
    cd ..
    
    echo -e "${GREEN}✓${NC} Project dependencies installed"
    echo ""
}

# Install Tauri CLI
install_tauri_cli() {
    echo "Installing Tauri CLI..."
    source "$HOME/.cargo/env" 2>/dev/null || true
    if ! command_exists cargo-tauri; then
        cargo install tauri-cli
    fi
    echo -e "${GREEN}✓${NC} Tauri CLI installed"
    echo ""
}

# Main installation
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
