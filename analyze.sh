#!/bin/bash
# S32G3 Rust binary analysis script

# Determine target
TARGET="aarch64-unknown-none-softfloat"  # Default target
if rustup target list --installed | grep -q "aarch64-unknown-none"; then
    TARGET="aarch64-unknown-none"
elif rustup target list --installed | grep -q "aarch64-unknown-none-softfloat"; then
    TARGET="aarch64-unknown-none-softfloat"
elif rustup target list --installed | grep -q "aarch64-unknown-linux-gnu"; then
    TARGET="aarch64-unknown-linux-gnu"
fi

# Find the appropriate tools
if command -v aarch64-none-elf-objdump &> /dev/null; then
    OBJDUMP="aarch64-none-elf-objdump"
    READELF="aarch64-none-elf-readelf"
    NM="aarch64-none-elf-nm"
    SIZE="aarch64-none-elf-size"
elif command -v aarch64-linux-gnu-objdump &> /dev/null; then
    OBJDUMP="aarch64-linux-gnu-objdump"
    READELF="aarch64-linux-gnu-readelf"
    NM="aarch64-linux-gnu-nm"
    SIZE="aarch64-linux-gnu-size"
else
    echo "ERROR: No suitable toolchain found. Please install one of:"
    echo "  - aarch64-none-elf-* (ARM embedded toolchain)"
    echo "  - aarch64-linux-gnu-* (GNU toolchain)"
    exit 1
fi

# Source and output files
ELF_FILE="target/$TARGET/release/freertos-s32g3-rust"
BIN_FILE="s32g3-rust.bin"
OUTPUT_DIR="analysis"

# Create output directory
mkdir -p $OUTPUT_DIR

echo "Analyzing S32G3 Rust binary..."
echo "Using toolchain: $OBJDUMP"

# Disassemble main sections
echo "Creating disassembly..."
$OBJDUMP -d $ELF_FILE > $OUTPUT_DIR/disassembly.txt

# Disassemble with source code
echo "Creating disassembly with source..."
$OBJDUMP -S $ELF_FILE > $OUTPUT_DIR/disassembly_with_source.txt

# Specifically check the boot section
echo "Analyzing boot section..."
$OBJDUMP -d -j .text.boot $ELF_FILE > $OUTPUT_DIR/boot_section.txt 2>/dev/null
if [ $? -ne 0 ]; then
    # Try alternative section name
    $OBJDUMP -d -j .boot $ELF_FILE > $OUTPUT_DIR/boot_section.txt 2>/dev/null
fi

# Get header information
echo "Extracting ELF header information..."
$READELF -a $ELF_FILE > $OUTPUT_DIR/elf_info.txt

# Get symbol table
echo "Extracting symbol table..."
$NM $ELF_FILE | sort > $OUTPUT_DIR/symbols.txt

# Get sections and sizes
echo "Analyzing section sizes..."
$SIZE -A $ELF_FILE > $OUTPUT_DIR/section_sizes.txt

# Disassemble the binary
echo "Disassembling binary file..."
$OBJDUMP -D -b binary -m aarch64 --adjust-vma=0xE0000000 $BIN_FILE > $OUTPUT_DIR/binary_disassembly.txt

# Extract first 1KB as hex dump
echo "Creating hex dump of first 1KB..."
hexdump -C -n 1024 $BIN_FILE > $OUTPUT_DIR/hexdump.txt

echo "Analysis complete! Results in $OUTPUT_DIR directory:"
ls -la $OUTPUT_DIR

echo "Key files to examine:"
echo " - $OUTPUT_DIR/boot_section.txt - Boot/startup code"
echo " - $OUTPUT_DIR/disassembly.txt - Complete disassembly"
echo " - $OUTPUT_DIR/section_sizes.txt - Memory usage by section"