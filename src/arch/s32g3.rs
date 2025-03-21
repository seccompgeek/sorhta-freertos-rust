// S32G3 specific memory-mapped registers and configuration
// Based on S32G3 Reference Manual

use core::{arch, ptr::{read_volatile, write_volatile}};
use cortex_a::asm;

use crate::drivers::uart;

use super::{enable_interrupts, gic};

// S32G3 base addresses for key peripherals
pub const UART_BASE: usize = 0x401C8000;  // LinFLEX UART0 base address
pub const GIC_DIST_BASE: usize = 0x50800000;  // GIC-500 Distributor
pub const GIC_CPU_BASE: usize = 0x50880000;   // GIC-500 CPU Interface

// LinFLEX UART register offsets
pub const LINFLEX_LINCR1: usize = 0x00;     // LIN Control Register 1
pub const LINFLEX_LINSR: usize = 0x8;      // LIN Status Register
pub const LINFLEX_UARTCR: usize = 0x10;     // UART Mode Control Register
pub const LINFLEX_UARTSR: usize = 0x14;     // UART Mode Status Register
pub const LINFLEX_LINIBRR: usize = 0x28;    // LIN Integer Baud Rate Register
pub const LINFLEX_LINFBRR: usize = 0x24;    // LIN Fractional Baud Rate Register
pub const LINFLEX_BDRL: usize = 0x38;       // Buffer Data Register Least Significant
pub const LINFLEX_UARTPTO: usize = 0x50;    // UART Preset Timeout Register

// LinFLEX UART register bit definitions
pub const LINCR1_INIT: u32 = 1 << 0;        // Initialization Mode
pub const LINCR1_MME: u32 = 1 << 4;         // Master Mode Enable
pub const LINSR_LINS_MASK: u32 = 0x0000F000;       // LIN State Field Mask
pub const LINSR_LINS_INITMODE: u32 = 0x00001000;   // Initialization Mode
pub const LINSR_LINS_RX_TX_MODE: u32 = 0x00008000;
pub const UARTCR_UART: u32 = 1 << 0;        // UART Mode
pub const UARTCR_WL0: u32 = 1 << 1;         // Word Length bit 0 (8-bit)
pub const UARTCR_PC0: u32 = 1 << 3;         // Parity Control bit 0
pub const UARTCR_PC1: u32 = 1 << 6;         // Parity Control bit 1
pub const UARTCR_TXEN: u32 = 1 << 4;       // Transmitter Enable
pub const UARTCR_RXEN: u32 = 1 << 5;       // Receiver Enable
pub const UARTCR_TFBM: u32 = 1 << 8;        // Tx FIFO Buffer Mode
pub const UARTCR_RFBM: u32 = 1 << 9;        // Rx FIFO Buffer Mode
pub const UARTCR_ROSE: u32 = 1 << 23;       // Reduced Oversampling Enable
pub const UARTCR_TFC: u32 = ((0xFFFFFFFF) << (13)) & (0xFFFFFFFF >> (32 - 1 - (15)));         // Tx FIFO Counter mask
pub const UARTSR_DTF: u32 = 1 << 1;         // Data Transmission Completed Flag

// LinFLEX UART configuration values
pub const UART_CLOCK_HZ: u32 = 125000000;  // 80 MHz UART clock
pub const UART_BAUD_RATE: u32 = 115200;     // Default baud rate
pub const LDIV_MULTIPLIER: u32 = 16;        // Default LIN divider multiplier
pub const UARTCR_OSR_SHIFT: usize = 24;
pub const UARTCR_OSR_WIDTH: usize = 4;
pub const CONSOLE_T_BASE: usize = 32;
pub const CONSOLE_T_PUTC: usize = 8;
pub const CONSOLE_T_FLUSH: usize = 12;
pub const CONSOLE_T_GETC: usize = 24;
pub const CONSOLE_T_FLAGS: usize = 4;
pub const CONSOLE_FLAG_BOOT: usize = 1; 
pub const CONSOLE_FLAG_RUNTIME: usize = 1 << 1;
pub const CONSOLE_FLAG_CRASH: usize = 1 << 2;

// Memory-mapped timer constants
pub const S32G_STM0_BASE: usize = 0x40054000;  // System Timer Module 0
pub const S32G_STM_CR: usize = 0x00;      // Control Register offset
pub const S32G_STM_CNT: usize = 0x04;     // Count Register offset
pub const S32G_STM_CMP0: usize = 0x10;    // Compare Register 0 offset

// Clock configuration
pub const S32G_CLOCK_FREQ: u64 = 80_000_000;  // 80 MHz system clock (approximate)

pub mod timer {
    use core::sync::atomic::{AtomicU64, Ordering};
    use super::*;

    // System tick counter
    static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

    // Initialize the system timer
    pub fn init() {
        unsafe {
            // Access STM0 registers
            let stm_base = S32G_STM0_BASE as *mut u32;
            
            // Configure STM0 with a 1ms tick rate
            // Enable timer, set to free-running mode
            write_volatile(stm_base.add(S32G_STM_CR / 4), 0x1);
            
            // Set initial compare value
            write_volatile(stm_base.add(S32G_STM_CMP0 / 4), S32G_CLOCK_FREQ as u32 / 1000);
        }
    }

    // Read the system timer counter
    pub fn get_system_ticks() -> u64 {
        SYSTEM_TICKS.load(Ordering::Relaxed)
    }

    // Update the system timer (would be called by timer interrupt handler)
    pub fn update_system_ticks(ticks: u64) {
        SYSTEM_TICKS.store(ticks, Ordering::Relaxed);
    }

    // Increment the system timer (called by timer interrupt handler)
    pub fn increment_system_ticks() {
        SYSTEM_TICKS.fetch_add(1, Ordering::Relaxed);
    }

    // Read raw STM counter value
    pub fn get_raw_counter() -> u32 {
        unsafe {
            let stm_base = S32G_STM0_BASE as *const u32;
            read_volatile(stm_base.add(S32G_STM_CNT / 4))
        }
    }

    // Delay for a specified number of microseconds
    pub fn delay_us(us: u32) {
        // More accurate delay based on STM counter
        let start = get_raw_counter();
        let ticks_to_wait = (S32G_CLOCK_FREQ as u32 / 1_000_000) * us;
        
        while get_raw_counter().wrapping_sub(start) < ticks_to_wait {
            asm::nop();
        }
    }

    // Delay for a specified number of milliseconds
    pub fn delay_ms(ms: u32) {
        for _ in 0..ms {
            delay_us(1000);
        }
    }
}

// Initialize S32G3 peripheral clocks and basic hardware
pub fn init() {
    // Initialize system timer
    gic::init();
    uart::init();
    enable_interrupts();
    // In a full implementation, would initialize other S32G3-specific
    // hardware like clocks, GPIOs, etc.
}