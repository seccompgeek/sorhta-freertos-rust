#!/bin/bash
set -e

echo "Setting up S32G3 Rust Bare-metal Development Environment"
echo "========================================================"

# Check if rustup is installed
if ! command -v rustup &> /dev/null; then
    echo "rustup not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Install nightly toolchain
echo "Installing nightly Rust toolchain..."
rustup default nightly

# Install preferred target
echo "Installing AArch64 targets..."
rustup target add aarch64-unknown-none-softfloat

# Try to install aarch64-unknown-none if possible, but don't fail if it's not available
rustup target add aarch64-unknown-none || echo "aarch64-unknown-none not available, using aarch64-unknown-none-softfloat instead"

# Check if we have aarch64 toolchain for objcopy
if ! command -v aarch64-none-elf-objcopy &> /dev/null && ! command -v aarch64-linux-gnu-objcopy &> /dev/null; then
    echo "No AArch64 toolchain found for objcopy."
    
    # Try to install based on platform
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get &> /dev/null; then
            echo "Detected Debian/Ubuntu. Installing toolchain..."
            sudo apt-get update
            sudo apt-get install -y gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu
        elif command -v dnf &> /dev/null; then
            echo "Detected Fedora/RHEL. Installing toolchain..."
            sudo dnf install -y aarch64-linux-gnu-gcc aarch64-linux-gnu-binutils
        else
            echo "Unsupported Linux distribution. Please install AArch64 toolchain manually."
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        if command -v brew &> /dev/null; then
            echo "Detected macOS with Homebrew. Installing toolchain..."
            brew install --cask gcc-arm-embedded
        else
            echo "Please install Homebrew and then run: brew install --cask gcc-arm-embedded"
        fi
    else
        echo "Unsupported OS. Please install AArch64 toolchain manually."
    fi
fi

echo "Making scripts executable..."
chmod +x build.sh
chmod +x analyze.sh

# Verify installation
if command -v aarch64-none-elf-objcopy &> /dev/null; then
    OBJCOPY="aarch64-none-elf-objcopy"
elif command -v aarch64-linux-gnu-objcopy &> /dev/null; then
    OBJCOPY="aarch64-linux-gnu-objcopy"
elif command -v llvm-objcopy &> /dev/null; then
    OBJCOPY="llvm-objcopy"
else
    echo "WARNING: No suitable objcopy found. You may need to install it manually."
    echo "For Ubuntu/Debian: sudo apt-get install binutils-aarch64-linux-gnu"
    echo "For Fedora/RHEL: sudo dnf install aarch64-linux-gnu-binutils"
    echo "For macOS: brew install --cask gcc-arm-embedded"
    exit 1
fi

echo "Found objcopy: $OBJCOPY"
echo ""
echo "Setup complete! You can now build the project with:"
echo "./build.sh"
echo ""
echo "And analyze the binary with:"
echo "./analyze.sh"