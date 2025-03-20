// AArch64 exception vectors and handlers
// See ARM Architecture Reference Manual ARMv8 for details on exception model

use core::arch::global_asm;
use crate::println;
use crate::arch::gic;
use core::arch::asm;

// Define exception vector table for AArch64
global_asm!(
    ".section .text.exceptions",
    ".align 11",  // 2048-byte alignment for vector table
    
    // Vector table must be 2048 bytes, with 128-byte spacing
    ".global exception_vector_table",
    "exception_vector_table:",
    
    // Current EL with SP0
    "// Current EL with SP0",
    ".align 7",  // 128-byte alignment for each entry
    "b el1_sp0_sync",        // Synchronous
    ".align 7",
    "b el1_sp0_irq",         // IRQ
    ".align 7",
    "b el1_sp0_fiq",         // FIQ
    ".align 7",
    "b el1_sp0_serror",      // SError
    
    // Current EL with SPx
    "// Current EL with SPx",
    ".align 7",
    "b el1_sync",            // Synchronous
    ".align 7",
    "b el1_irq",             // IRQ
    ".align 7",
    "b el1_fiq",             // FIQ
    ".align 7",
    "b el1_serror",          // SError
    
    // Lower EL using AArch64
    "// Lower EL using AArch64",
    ".align 7",
    "b lower_el_aarch64_sync", // Synchronous
    ".align 7",
    "b lower_el_aarch64_irq",  // IRQ
    ".align 7",
    "b lower_el_aarch64_fiq",  // FIQ
    ".align 7",
    "b lower_el_aarch64_serror", // SError
    
    // Lower EL using AArch32
    "// Lower EL using AArch32",
    ".align 7",
    "b lower_el_aarch32_sync", // Synchronous
    ".align 7",
    "b lower_el_aarch32_irq",  // IRQ
    ".align 7",
    "b lower_el_aarch32_fiq",  // FIQ
    ".align 7",
    "b lower_el_aarch32_serror", // SError
    
    // Exception handlers
    "el1_sp0_sync:",
    "   bl exception_handler_sp0_sync",
    "   eret",
    
    "el1_sp0_irq:",
    "   bl exception_handler_sp0_irq",
    "   eret",
    
    "el1_sp0_fiq:",
    "   bl exception_handler_sp0_fiq",
    "   eret",
    
    "el1_sp0_serror:",
    "   bl exception_handler_sp0_serror",
    "   eret",
    
    "el1_sync:",
    "   bl exception_handler_sync",
    "   eret",
    
    "el1_irq:",
    "   bl exception_handler_irq",
    "   eret",
    
    "el1_fiq:",
    "   bl exception_handler_fiq",
    "   eret",
    
    "el1_serror:",
    "   bl exception_handler_serror",
    "   eret",
    
    "lower_el_aarch64_sync:",
    "   bl exception_handler_lower_sync",
    "   eret",
    
    "lower_el_aarch64_irq:",
    "   bl exception_handler_lower_irq",
    "   eret",
    
    "lower_el_aarch64_fiq:",
    "   bl exception_handler_lower_fiq",
    "   eret",
    
    "lower_el_aarch64_serror:",
    "   bl exception_handler_lower_serror",
    "   eret",
    
    "lower_el_aarch32_sync:",
    "   bl exception_handler_lower32_sync",
    "   eret",
    
    "lower_el_aarch32_irq:",
    "   bl exception_handler_lower32_irq",
    "   eret",
    
    "lower_el_aarch32_fiq:",
    "   bl exception_handler_lower32_fiq",
    "   eret",
    
    "lower_el_aarch32_serror:",
    "   bl exception_handler_lower32_serror",
    "   eret",
);

// Exception handler typedefs
pub type ExceptionHandler = fn() -> ();

// Initialize exception vectors
pub fn init_vectors() {
    unsafe {
        // Set VBAR_EL1 to point to our exception vector table
        let vbar_el1 = &exception_vector_table as *const u64;
        asm!(
            "msr vbar_el1, {x}",
            x = in(reg) vbar_el1,
            options(nostack)
        );
    }
}

#[no_mangle]
extern "C" fn exception_handler_irq() {
    // Get interrupt ID from GIC
    let irq_id = gic::get_interrupt_id();
    
    // Check for spurious interrupt
    if irq_id == 1023 {
        return;
    }
    
    // Handle the specific interrupt
    handle_interrupt(irq_id);
    
    // Signal end of interrupt to GIC
    gic::end_of_interrupt(irq_id);
}

// IRQ handler for SP0 mode
#[no_mangle]
extern "C" fn exception_handler_sp0_irq() {
    exception_handler_irq();
}

// IRQ handler for lower EL AArch64
#[no_mangle]
extern "C" fn exception_handler_lower_irq() {
    exception_handler_irq();
}

// IRQ handler for lower EL AArch32
#[no_mangle]
extern "C" fn exception_handler_lower32_irq() {
    exception_handler_irq();
}

// FIQ handler
#[no_mangle]
extern "C" fn exception_handler_fiq() {
    println!("FIQ exception occurred");
}

// SP0 FIQ handler
#[no_mangle]
extern "C" fn exception_handler_sp0_fiq() {
    exception_handler_fiq();
}

// Lower EL FIQ handler (AArch64)
#[no_mangle]
extern "C" fn exception_handler_lower_fiq() {
    exception_handler_fiq();
}

// Lower EL FIQ handler (AArch32)
#[no_mangle]
extern "C" fn exception_handler_lower32_fiq() {
    exception_handler_fiq();
}

// Synchronous exception handler
#[no_mangle]
extern "C" fn exception_handler_sync() {
    // Read exception syndrome register
    let esr: u64;
    unsafe {
        asm!(
            "mrs {x}, esr_el1",
            x = out(reg) esr,
            options(nostack)
        );
    }
    
    // Extract exception class (EC) from ESR
    let ec = (esr >> 26) & 0x3F;
    
    match ec {
        0x15 => println!("SVC instruction execution in AArch64"),
        0x24 => println!("Data abort from current EL"),
        _ => println!("Synchronous exception: ESR = 0x{:X}", esr),
    }
}

// SP0 synchronous exception handler
#[no_mangle]
extern "C" fn exception_handler_sp0_sync() {
    exception_handler_sync();
}

// Lower EL synchronous exception handler (AArch64)
#[no_mangle]
extern "C" fn exception_handler_lower_sync() {
    exception_handler_sync();
}

// Lower EL synchronous exception handler (AArch32)
#[no_mangle]
extern "C" fn exception_handler_lower32_sync() {
    exception_handler_sync();
}

// SError handler
#[no_mangle]
extern "C" fn exception_handler_serror() {
    println!("SError exception occurred");
}

// SP0 SError handler
#[no_mangle]
extern "C" fn exception_handler_sp0_serror() {
    exception_handler_serror();
}

// Lower EL SError handler (AArch64)
#[no_mangle]
extern "C" fn exception_handler_lower_serror() {
    exception_handler_serror();
}

// Lower EL SError handler (AArch32)
#[no_mangle]
extern "C" fn exception_handler_lower32_serror() {
    exception_handler_serror();
}

// Vector base address (defined in assembly)
extern "C" {
    static exception_vector_table: u64;
}

// Handle specific interrupt based on ID
fn handle_interrupt(irq_id: u32) {
    match irq_id {
        // UART interrupt
        33 => {
            println!("UART Interrupt received");
            // Handle UART interrupt
        },
        
        // Timer interrupt
        27 => {
            println!("Timer Interrupt received");
            // Handle timer interrupt
        },
        
        // Generic interrupt handler for other IRQs
        _ => {
            println!("Received IRQ: {}", irq_id);
        }
    }
}