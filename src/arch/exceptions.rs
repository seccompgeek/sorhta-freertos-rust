// AArch64 exception vectors and handlers
// See ARM Architecture Reference Manual ARMv8 for details on exception model

use core::arch::global_asm;
use core::arch::asm;
use crate::drivers::uart;
use crate::arch::gic;

// Define exception vector table for AArch64
global_asm!(
    // Ensure the section is correctly defined
    ".section .text.exceptions, \"ax\"",
    ".align 11",  // 2048-byte alignment for vector table
    
    // Vector table must be 2048 bytes
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
    "   // Save the current state",
    "   sub sp, sp, #16 * 17",  // Space for x0-x30, elr_el1, spsr_el1
    "   stp x0, x1, [sp, #16 * 0]",
    "   stp x2, x3, [sp, #16 * 1]",
    "   stp x4, x5, [sp, #16 * 2]",
    "   stp x6, x7, [sp, #16 * 3]",
    "   stp x8, x9, [sp, #16 * 4]",
    "   stp x10, x11, [sp, #16 * 5]",
    "   stp x12, x13, [sp, #16 * 6]",
    "   stp x14, x15, [sp, #16 * 7]",
    "   stp x16, x17, [sp, #16 * 8]",
    "   stp x18, x19, [sp, #16 * 9]",
    "   stp x20, x21, [sp, #16 * 10]",
    "   stp x22, x23, [sp, #16 * 11]",
    "   stp x24, x25, [sp, #16 * 12]",
    "   stp x26, x27, [sp, #16 * 13]",
    "   stp x28, x29, [sp, #16 * 14]",
    "   mrs x21, elr_el1",
    "   mrs x22, spsr_el1",
    "   stp x30, x21, [sp, #16 * 15]",
    "   str x22, [sp, #16 * 16]",
    "   bl exception_handler_sp0_sync",
    "   // Restore state",
    "   ldp x30, x21, [sp, #16 * 15]",
    "   ldr x22, [sp, #16 * 16]",
    "   msr elr_el1, x21",
    "   msr spsr_el1, x22",
    "   ldp x0, x1, [sp, #16 * 0]",
    "   ldp x2, x3, [sp, #16 * 1]",
    "   ldp x4, x5, [sp, #16 * 2]",
    "   ldp x6, x7, [sp, #16 * 3]",
    "   ldp x8, x9, [sp, #16 * 4]",
    "   ldp x10, x11, [sp, #16 * 5]",
    "   ldp x12, x13, [sp, #16 * 6]",
    "   ldp x14, x15, [sp, #16 * 7]",
    "   ldp x16, x17, [sp, #16 * 8]",
    "   ldp x18, x19, [sp, #16 * 9]",
    "   ldp x20, x21, [sp, #16 * 10]",
    "   ldp x22, x23, [sp, #16 * 11]",
    "   ldp x24, x25, [sp, #16 * 12]",
    "   ldp x26, x27, [sp, #16 * 13]",
    "   ldp x28, x29, [sp, #16 * 14]",
    "   add sp, sp, #16 * 17",
    "   eret",
    
    "el1_sp0_irq:",
    "   // Save the current state",
    "   sub sp, sp, #16 * 17",  // Space for x0-x30, elr_el1, spsr_el1
    "   stp x0, x1, [sp, #16 * 0]",
    "   stp x2, x3, [sp, #16 * 1]",
    "   stp x4, x5, [sp, #16 * 2]",
    "   stp x6, x7, [sp, #16 * 3]",
    "   stp x8, x9, [sp, #16 * 4]",
    "   stp x10, x11, [sp, #16 * 5]",
    "   stp x12, x13, [sp, #16 * 6]",
    "   stp x14, x15, [sp, #16 * 7]",
    "   stp x16, x17, [sp, #16 * 8]",
    "   stp x18, x19, [sp, #16 * 9]",
    "   stp x20, x21, [sp, #16 * 10]",
    "   stp x22, x23, [sp, #16 * 11]",
    "   stp x24, x25, [sp, #16 * 12]",
    "   stp x26, x27, [sp, #16 * 13]",
    "   stp x28, x29, [sp, #16 * 14]",
    "   mrs x21, elr_el1",
    "   mrs x22, spsr_el1",
    "   stp x30, x21, [sp, #16 * 15]",
    "   str x22, [sp, #16 * 16]",
    "   bl exception_handler_sp0_irq",
    "   // Restore state",
    "   ldp x30, x21, [sp, #16 * 15]",
    "   ldr x22, [sp, #16 * 16]",
    "   msr elr_el1, x21",
    "   msr spsr_el1, x22",
    "   ldp x0, x1, [sp, #16 * 0]",
    "   ldp x2, x3, [sp, #16 * 1]",
    "   ldp x4, x5, [sp, #16 * 2]",
    "   ldp x6, x7, [sp, #16 * 3]",
    "   ldp x8, x9, [sp, #16 * 4]",
    "   ldp x10, x11, [sp, #16 * 5]",
    "   ldp x12, x13, [sp, #16 * 6]",
    "   ldp x14, x15, [sp, #16 * 7]",
    "   ldp x16, x17, [sp, #16 * 8]",
    "   ldp x18, x19, [sp, #16 * 9]",
    "   ldp x20, x21, [sp, #16 * 10]",
    "   ldp x22, x23, [sp, #16 * 11]",
    "   ldp x24, x25, [sp, #16 * 12]",
    "   ldp x26, x27, [sp, #16 * 13]",
    "   ldp x28, x29, [sp, #16 * 14]",
    "   add sp, sp, #16 * 17",
    "   eret",
    
    "el1_sp0_fiq:",
    "   bl exception_handler_sp0_fiq",
    "   eret",
    
    "el1_sp0_serror:",
    "   bl exception_handler_sp0_serror",
    "   eret",
    
    "el1_sync:",
    "   // Save the current state",
    "   sub sp, sp, #16 * 17",
    "   stp x0, x1, [sp, #16 * 0]",
    "   stp x2, x3, [sp, #16 * 1]",
    "   stp x4, x5, [sp, #16 * 2]",
    "   stp x6, x7, [sp, #16 * 3]",
    "   stp x8, x9, [sp, #16 * 4]",
    "   stp x10, x11, [sp, #16 * 5]",
    "   stp x12, x13, [sp, #16 * 6]",
    "   stp x14, x15, [sp, #16 * 7]",
    "   stp x16, x17, [sp, #16 * 8]",
    "   stp x18, x19, [sp, #16 * 9]",
    "   stp x20, x21, [sp, #16 * 10]",
    "   stp x22, x23, [sp, #16 * 11]",
    "   stp x24, x25, [sp, #16 * 12]",
    "   stp x26, x27, [sp, #16 * 13]",
    "   stp x28, x29, [sp, #16 * 14]",
    "   mrs x21, elr_el1",
    "   mrs x22, spsr_el1",
    "   stp x30, x21, [sp, #16 * 15]",
    "   str x22, [sp, #16 * 16]",
    "   bl exception_handler_sync",
    "   // Restore state",
    "   ldp x30, x21, [sp, #16 * 15]",
    "   ldr x22, [sp, #16 * 16]",
    "   msr elr_el1, x21",
    "   msr spsr_el1, x22",
    "   ldp x0, x1, [sp, #16 * 0]",
    "   ldp x2, x3, [sp, #16 * 1]",
    "   ldp x4, x5, [sp, #16 * 2]",
    "   ldp x6, x7, [sp, #16 * 3]",
    "   ldp x8, x9, [sp, #16 * 4]",
    "   ldp x10, x11, [sp, #16 * 5]",
    "   ldp x12, x13, [sp, #16 * 6]",
    "   ldp x14, x15, [sp, #16 * 7]",
    "   ldp x16, x17, [sp, #16 * 8]",
    "   ldp x18, x19, [sp, #16 * 9]",
    "   ldp x20, x21, [sp, #16 * 10]",
    "   ldp x22, x23, [sp, #16 * 11]",
    "   ldp x24, x25, [sp, #16 * 12]",
    "   ldp x26, x27, [sp, #16 * 13]",
    "   ldp x28, x29, [sp, #16 * 14]",
    "   add sp, sp, #16 * 17",
    "   eret",
    
    "el1_irq:",
    "   // Save the current state",
    "   sub sp, sp, #16 * 17",
    "   stp x0, x1, [sp, #16 * 0]",
    "   stp x2, x3, [sp, #16 * 1]",
    "   stp x4, x5, [sp, #16 * 2]",
    "   stp x6, x7, [sp, #16 * 3]",
    "   stp x8, x9, [sp, #16 * 4]",
    "   stp x10, x11, [sp, #16 * 5]",
    "   stp x12, x13, [sp, #16 * 6]",
    "   stp x14, x15, [sp, #16 * 7]",
    "   stp x16, x17, [sp, #16 * 8]",
    "   stp x18, x19, [sp, #16 * 9]",
    "   stp x20, x21, [sp, #16 * 10]",
    "   stp x22, x23, [sp, #16 * 11]",
    "   stp x24, x25, [sp, #16 * 12]",
    "   stp x26, x27, [sp, #16 * 13]",
    "   stp x28, x29, [sp, #16 * 14]",
    "   mrs x21, elr_el1",
    "   mrs x22, spsr_el1",
    "   stp x30, x21, [sp, #16 * 15]",
    "   str x22, [sp, #16 * 16]",
    "   bl exception_handler_irq",
    "   // Restore state",
    "   ldp x30, x21, [sp, #16 * 15]",
    "   ldr x22, [sp, #16 * 16]",
    "   msr elr_el1, x21",
    "   msr spsr_el1, x22",
    "   ldp x0, x1, [sp, #16 * 0]",
    "   ldp x2, x3, [sp, #16 * 1]",
    "   ldp x4, x5, [sp, #16 * 2]",
    "   ldp x6, x7, [sp, #16 * 3]",
    "   ldp x8, x9, [sp, #16 * 4]",
    "   ldp x10, x11, [sp, #16 * 5]",
    "   ldp x12, x13, [sp, #16 * 6]",
    "   ldp x14, x15, [sp, #16 * 7]",
    "   ldp x16, x17, [sp, #16 * 8]",
    "   ldp x18, x19, [sp, #16 * 9]",
    "   ldp x20, x21, [sp, #16 * 10]",
    "   ldp x22, x23, [sp, #16 * 11]",
    "   ldp x24, x25, [sp, #16 * 12]",
    "   ldp x26, x27, [sp, #16 * 13]",
    "   ldp x28, x29, [sp, #16 * 14]",
    "   add sp, sp, #16 * 17",
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
    uart::puts("SP0 IRQ Exception\r\n");
    exception_handler_irq();
}

// IRQ handler for lower EL AArch64
#[no_mangle]
extern "C" fn exception_handler_lower_irq() {
    uart::puts("Lower AArch64 IRQ Exception\r\n");
    exception_handler_irq();
}

// IRQ handler for lower EL AArch32
#[no_mangle]
extern "C" fn exception_handler_lower32_irq() {
    uart::puts("Lower AArch32 IRQ Exception\r\n");
    exception_handler_irq();
}

// FIQ handler
#[no_mangle]
extern "C" fn exception_handler_fiq() {
    uart::puts("FIQ Exception\r\n");
}

// SP0 FIQ handler
#[no_mangle]
extern "C" fn exception_handler_sp0_fiq() {
    uart::puts("SP0 FIQ Exception\r\n");
}

// Lower EL FIQ handler (AArch64)
#[no_mangle]
extern "C" fn exception_handler_lower_fiq() {
    uart::puts("Lower AArch64 FIQ Exception\r\n");
}

// Lower EL FIQ handler (AArch32)
#[no_mangle]
extern "C" fn exception_handler_lower32_fiq() {
    uart::puts("Lower AArch32 FIQ Exception\r\n");
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
    
    // Print information about the exception
    uart::puts("Synchronous Exception: ESR=0x");
    print_hex(esr);
    uart::puts("\r\n");
    
    match ec {
        0x15 => uart::puts("SVC instruction execution in AArch64\r\n"),
        0x24 => uart::puts("Data abort from current EL\r\n"),
        _ => {
            uart::puts("Unknown exception class: 0x");
            print_hex(ec);
            uart::puts("\r\n");
        }
    }
}

// SP0 synchronous exception handler
#[no_mangle]
extern "C" fn exception_handler_sp0_sync() {
    uart::puts("SP0 Synchronous Exception\r\n");
    exception_handler_sync();
}

// Lower EL synchronous exception handler (AArch64)
#[no_mangle]
extern "C" fn exception_handler_lower_sync() {
    uart::puts("Lower AArch64 Synchronous Exception\r\n");
    exception_handler_sync();
}

// Lower EL synchronous exception handler (AArch32)
#[no_mangle]
extern "C" fn exception_handler_lower32_sync() {
    uart::puts("Lower AArch32 Synchronous Exception\r\n");
    exception_handler_sync();
}

// SError handler
#[no_mangle]
extern "C" fn exception_handler_serror() {
    uart::puts("SError Exception\r\n");
}

// SP0 SError handler
#[no_mangle]
extern "C" fn exception_handler_sp0_serror() {
    uart::puts("SP0 SError Exception\r\n");
}

// Lower EL SError handler (AArch64)
#[no_mangle]
extern "C" fn exception_handler_lower_serror() {
    uart::puts("Lower AArch64 SError Exception\r\n");
}

// Lower EL SError handler (AArch32)
#[no_mangle]
extern "C" fn exception_handler_lower32_serror() {
    uart::puts("Lower AArch32 SError Exception\r\n");
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
            uart::puts("UART Interrupt received\r\n");
            // Handle UART interrupt
        },
        
        // Timer interrupt
        27 => {
            uart::puts("Timer Interrupt received\r\n");
            // Handle timer interrupt
        },
        
        // Generic interrupt handler for other IRQs
        _ => {
            uart::puts("Received IRQ: ");
            print_hex(irq_id as u64);
            uart::puts("\r\n");
        }
    }
}

// Helper function to print hex values
fn print_hex(value: u64) {
    const HEX_CHARS: [u8; 16] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', 
                                b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F'];
    
    uart::puts("0x");
    
    for i in (0..16).rev() {
        let digit = ((value >> (i * 4)) & 0xF) as usize;
        uart::putc(HEX_CHARS[digit]);
    }
}