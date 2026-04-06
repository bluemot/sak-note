#!/bin/bash
# SAK Editor - Build Script for Linux/macOS
# This script builds the release version of SAK Editor

# Show help
if [[ "$1" == "--help" || "$1" == "-h" || "$1" == "help" ]]; then
    echo ""
    echo "Usage: ./build.sh [OPTIONS]"
    echo ""
    echo "Builds SAK Editor release version:"
    echo "  1. Cleans previous builds"
    echo "  2. Builds frontend (npm run build)"
    echo "  3. Builds Tauri release"
    echo "  4. Shows output locations"
    echo ""
    echo "Options:"
    echo "  -h, --help, help    Show this help message"
    echo ""
    echo "Prerequisites:"
    echo "  Run './install.sh' first to install dependencies"
    echo ""
    echo "Output:"
    echo "  - Linux: src-tauri/target/release/bundle/"
    echo "  - macOS: src-tauri/target/release/bundle/"
    echo ""
    exit 0
fi

set -e

echo "=========================================="
echo "SAK Editor - Release Build"
echo "=========================================="
echo ""
echo "Run './build.sh help' for usage info"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Check prerequisites
echo "Checking prerequisites..."

if ! command -v node > /dev/null 2>&1; then
    echo -e "${RED}✗${NC} Node.js not found. Please run ./install.sh first"
    exit 1
fi

if ! command -v cargo > /dev/null 2>&1; then
    echo -e "${RED}✗${NC} Rust not found. Please run ./install.sh first"
    exit 1
fi

echo -e "${GREEN}✓${NC} Prerequisites met"
echo ""

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf src-frontend/dist
rm -rf src-tauri/target/release

echo -e "${GREEN}✓${NC} Clean complete"
echo ""

# Build frontend
echo -e "${BLUE}Building frontend...${NC}"
cd src-frontend
npm run build
if [ $? -ne 0 ]; then
    echo -e "${RED}✗${NC} Frontend build failed"
    exit 1
fi
echo -e "${GREEN}✓${NC} Frontend build complete"
echo ""

# Build Tauri release
echo -e "${BLUE}Building Tauri release...${NC}"
cd ../src-tauri

# Source cargo env
source "$HOME/.cargo/env" 2> /dev/null || true

# Limit parallel jobs to avoid "Too many open files" error
# See: https://github.com/rust-lang/cargo/issues/10455
export CARGO_BUILD_JOBS=4
echo "Using CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS to limit file descriptors"

cargo tauri build
if [ $? -ne 0 ]; then
    echo -e "${RED}✗${NC} Tauri build failed"
    exit 1
fi
echo -e "${GREEN}✓${NC} Tauri build complete"
echo ""

# Find the built executable
echo "Locating built executable..."
RELEASE_DIR="target/release"

if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    APP_PATH="${RELEASE_DIR}/bundle/macos/SAK Editor.app"
    if [ -d "$APP_PATH" ]; then
        echo -e "${GREEN}✓${NC} macOS app bundle: $APP_PATH"
        echo ""
        echo "To run: open \"$APP_PATH\""
    fi
    DMG_PATH=$(find ${RELEASE_DIR}/bundle -name "*.dmg" 2>/dev/null | head -1)
    if [ -n "$DMG_PATH" ]; then
        echo -e "${GREEN}✓${NC} DMG installer: $DMG_PATH"
    fi
else
    # Linux
    APP_IMAGE=$(find ${RELEASE_DIR}/bundle -name "*.AppImage" 2>/dev/null | head -1)
    DEB_PKG=$(find ${RELEASE_DIR}/bundle -name "*.deb" 2>/dev/null | head -1)
    RPM_PKG=$(find ${RELEASE_DIR}/bundle -name "*.rpm" 2>/dev/null | head -1)
    
    if [ -n "$APP_IMAGE" ]; then
        echo -e "${GREEN}✓${NC} AppImage: $APP_IMAGE"
    fi
    if [ -n "$DEB_PKG" ]; then
        echo -e "${GREEN}✓${NC} DEB package: $DEB_PKG"
    fi
    if [ -n "$RPM_PKG" ]; then
        echo -e "${GREEN}✓${NC} RPM package: $RPM_PKG"
    fi
    
    # Standalone binary
    if [ -f "${RELEASE_DIR}/sak-editor" ]; then
        echo -e "${GREEN}✓${NC} Standalone binary: ${RELEASE_DIR}/sak-editor"
        echo ""
        echo "To run: ./${RELEASE_DIR}/sak-editor"
    fi
fi

echo ""
echo "=========================================="
echo -e "${GREEN}Build complete!${NC}"
echo "=========================================="
echo ""
echo "Release artifacts are in: src-tauri/target/release/bundle/"
echo ""
