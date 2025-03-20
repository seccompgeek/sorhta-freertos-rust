# Rust Bare-metal Port for S32G3 Cortex-A

This project implements a bare-metal application for the NXP S32G3 board's Cortex-A53 cores, written in Rust. The implementation is designed to be loaded by ARM Trusted Firmware (ATF).

## S32G3 Technical Details

The NXP S32G3 is an automotive-grade processor with:
- Quad Cortex-A53 cores (application cores)
- Three Cortex-M7 cores (real-time cores)
- ASIL-D safety features
- Hardware security features
- Advanced networking capabilities

This port targets the Cortex-A53 cores and creates a binary that can be loaded by ARM Trusted Firmware (ATF) at address 0xE0000000.

## Features

- AArch64 boot code with proper stack and BSS initialization
- S32G3-specific timer implementation for accurate timing
- Simple UART driver for console output
- "Hello World" application that prints to the console 
- Heap allocation support with linked_list_allocator
- Runs on Cortex-A53 cores without MMU configuration
- Proper panic handler for debugging

## Building

### Quick Setup

For a quick setup of all dependencies, run the provided setup script:

```bash
# Make the script executable
chmod +x setup.sh

# Run the setup script
./setup.sh
```

This will install Rust (if needed), set up the nightly toolchain, add the necessary AArch64 target, and attempt to install the required cross-compiler tools.

### Manual Setup

If you prefer to set things up manually, follow these steps:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install nightly toolchain
rustup default nightly

# Add AArch64 target (one of these options)
rustup target add aarch64-unknown-none-softfloat
# OR
rustup target add aarch64-unknown-none

# Install cross-compiler toolchain for objcopy
# On Ubuntu/Debian:
sudo apt-get install gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu
# On Fedora/RHEL:
sudo dnf install aarch64-linux-gnu-gcc aarch64-linux-gnu-binutils
# On macOS with Homebrew:
brew install --cask gcc-arm-embedded
```

### Building the Project

Once the dependencies are set up, build the project:

```bash
# Make the build script executable
chmod +x build.sh

# Build the project
./build.sh
```

This will produce a binary file `s32g3-rust.bin` that can be loaded at address 0xE0000000 by ATF.

## Analyzing the Binary

To analyze the compiled binary, use the provided analysis script:

```bash
# Make the script executable
chmod +x analyze.sh

# Run the analysis
./analyze.sh
```

This will create an `analysis` directory with various files for examining the compiled code, including:
- Disassembly with and without source code
- Boot section analysis
- Symbol table and section sizes
- ELF header information

## Project Structure

- `src/main.rs` - Entry point and kernel initialization
- `src/arch/` - Architecture-specific code
  - `aarch64.rs` - AArch64 specific functions
  - `s32g3.rs` - S32G3 SoC specific code
- `src/drivers/` - Hardware drivers
  - `uart.rs` - UART driver for console output
- `build.sh` - Build script to compile and create binary
- `analyze.sh` - Script to analyze the compiled binary
- `link.ld` - Linker script defining memory layout

## Memory Map

The S32G3 features the following memory layout when used with ARM Trusted Firmware:

```
0x34000000 - 0x3FFFFFFF: SRAM (64 MB)
0x80000000 - 0xFFFFFFFF: DRAM (2 GB)
```

Our application is loaded by ATF at address 0xE0000000 in DRAM.

## Future Enhancements

- MMU configuration and virtual memory
- Interrupt controller (GIC-500) setup for interrupt handling
- Multi-core support (core synchronization)
- Additional peripheral drivers
- FreeRTOS integration

## License

This project is licensed under the MIT License - see the LICENSE file for details.