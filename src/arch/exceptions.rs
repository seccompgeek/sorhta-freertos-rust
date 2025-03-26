use core::arch::asm;

use alloc::format;

// Exception handlers for S32G3 Rust OS
use crate::drivers::uart;
use crate::gic;

// Exception handler implementations in Rust

#[no_mangle]
pub extern "C" fn handle_el1_sync_exception() {
    let esr: u64;
    let far: u64;
    
    // Read the exception registers
    unsafe {
        asm!("mrs {}, esr_el1", out(reg) esr);
        asm!("mrs {}, far_el1", out(reg) far);
    }
    
    let ec = (esr >> 26) & 0x3F;
    let iss = esr & 0x1FFFFFF;
    
    uart::puts(&format!("EL1 synchronous exception: ESR=0x{:x} (EC=0x{:x}, ISS=0x{:x}), FAR=0x{:x}\n",
                     esr, ec, iss, far));
    
    // Handle specific exception types
    match ec {
        0x15 => {
            // SVC instruction from same EL
            uart::puts("SVC from EL1 not supported\n");
        },
        0x17 => {
            // SMC instruction
            uart::puts("SMC from EL1\n");
            // Call SMC handler
        },
        0x21 => {
            // Instruction abort from EL1
            uart::puts("Instruction abort from EL1\n");
        },
        0x25 => {
            // Data abort from EL1
            uart::puts("Data abort from EL1\n");
        },
        _ => {
            uart::puts(&format!("Unhandled EC: 0x{:x}\n", ec));
        }
    }
}

#[no_mangle]
pub extern "C" fn handle_el1_irq() {
    // uart::puts("EL1 IRQ received\n");
    // gic::handle();
}

#[no_mangle]
pub extern "C" fn handle_el1_fiq() {
    uart::puts("EL1 FIQ received\n");
    // Handle FIQ
}

#[no_mangle]
pub extern "C" fn handle_el1_serror() {
    uart::puts("EL1 SError received\n");
    
    let esr: u64;
    unsafe {
        asm!("mrs {}, esr_el1", out(reg) esr);
    }
    
    uart::puts(&format!("System Error ESR=0x{:x}\n", esr));
}

#[no_mangle]
pub extern "C" fn handle_el0_irq() {
    // uart::puts("EL0 IRQ received\n");
    // gic::handle_irq();
}

#[no_mangle]
pub extern "C" fn handle_el0_fiq() {
    uart::puts("EL0 FIQ received\n");
    // Handle FIQ
}

#[no_mangle]
pub extern "C" fn handle_el0_serror() {
    uart::puts("EL0 SError received\n");
    
    let esr: u64;
    unsafe {
        asm!("mrs {}, esr_el1", out(reg) esr);
    }
    
    uart::puts(&format!("System Error ESR=0x{:x}\n", esr));
}

// AArch32 exception handlers (minimal implementation)
#[no_mangle]
pub extern "C" fn handle_el0_sync_a32() {
    uart::puts("EL0 AArch32 synchronous exception - not supported\n");
}

#[no_mangle]
pub extern "C" fn handle_el0_irq_a32() {
    uart::puts("EL0 AArch32 IRQ - not supported\n");
}

#[no_mangle]
pub extern "C" fn handle_el0_fiq_a32() {
    uart::puts("EL0 AArch32 FIQ - not supported\n");
}

#[no_mangle]
pub extern "C" fn handle_el0_serror_a32() {
    uart::puts("EL0 AArch32 SError - not supported\n");
}