#!/bin/bash
set -e

# Determine which target to use
TARGET="aarch64-unknown-none-softfloat"  # Default target

# Check if aarch64-unknown-none is installed
if rustup target list --installed | grep -q "aarch64-unknown-none"; then
    TARGET="aarch64-unknown-none"
elif rustup target list --installed | grep -q "aarch64-unknown-none-softfloat"; then
    TARGET="aarch64-unknown-none-softfloat"
elif rustup target list --installed | grep -q "aarch64-unknown-linux-gnu"; then
    TARGET="aarch64-unknown-linux-gnu"
else
    echo "No suitable target found. Installing aarch64-unknown-none-softfloat..."
    rustup target add aarch64-unknown-none-softfloat
fi

echo "Building for target: $TARGET"

# Build the project
cargo build --release --target $TARGET

# Path to built ELF
ELF_PATH="target/$TARGET/release/freertos-s32g3-rust"

# Find available objcopy tool
if command -v aarch64-none-elf-objcopy &> /dev/null; then
    OBJCOPY="aarch64-none-elf-objcopy"
elif command -v aarch64-linux-gnu-objcopy &> /dev/null; then
    OBJCOPY="aarch64-linux-gnu-objcopy"
elif command -v llvm-objcopy &> /dev/null; then
    OBJCOPY="llvm-objcopy"
else
    echo "ERROR: No suitable objcopy tool found. Please install one of:"
    echo "  - aarch64-none-elf-objcopy (ARM embedded toolchain)"
    echo "  - aarch64-linux-gnu-objcopy (GNU toolchain)"
    echo "  - llvm-objcopy (LLVM toolchain)"
    exit 1
fi

echo "Using objcopy tool: $OBJCOPY"

# Convert ELF to binary
$OBJCOPY -O binary $ELF_PATH s32g3-rust.bin

# Generate information about the binary
echo "Binary information:"
ls -la s32g3-rust.bin

# Create a hex dump for inspection (first 64 bytes)
echo -e "\nFirst 64 bytes of binary:"
hexdump -C -n 64 s32g3-rust.bin

echo -e "\nBuild complete. Use s32g3-rust.bin for loading via AT-F at address 0xE0000000"
echo "To disassemble: $OBJCOPY -d $ELF_PATH"