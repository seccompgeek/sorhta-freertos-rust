// AArch64 exception vectors and handlers
// See ARM Architecture Reference Manual ARMv8 for details on exception model

use core::arch::global_asm;
use core::arch::asm;
use crate::drivers::uart;
use crate::arch::gic;

// Define exception vector table for AArch64 with the U-Boot like approach
global_asm!(
    ".section .text.exceptions",
    ".align 11",  // 2048-byte alignment for vector table
    
    // Vector table must be 2048 bytes, with 128-byte spacing
    ".global exception_vector_table",
    "exception_vector_table:",
    
    // Current EL with SP0
    ".align 7",  // 128-byte alignment for each entry
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_sync_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_irq_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_fiq_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_error_handler",
    "b exception_exit",
    
    // Current EL with SPx
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_sync_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_irq_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_fiq_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_error_handler",
    "b exception_exit",
    
    // Lower EL using AArch64
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_sync_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_irq_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_fiq_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_error_handler",
    "b exception_exit",
    
    // Lower EL using AArch32
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_sync_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_irq_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_fiq_handler",
    "b exception_exit",
    
    ".align 7",
    "stp x29, x30, [sp, #-16]!",
    "bl _exception_entry",
    "bl do_error_handler",
    "b exception_exit",
    
    // Exception handling routines
    "_exception_entry:",
    "   stp x27, x28, [sp, #-16]!",
    "   stp x25, x26, [sp, #-16]!",
    "   stp x23, x24, [sp, #-16]!",
    "   stp x21, x22, [sp, #-16]!",
    "   stp x19, x20, [sp, #-16]!",
    "   stp x17, x18, [sp, #-16]!",
    "   stp x15, x16, [sp, #-16]!",
    "   stp x13, x14, [sp, #-16]!",
    "   stp x11, x12, [sp, #-16]!",
    "   stp x9, x10, [sp, #-16]!",
    "   stp x7, x8, [sp, #-16]!",
    "   stp x5, x6, [sp, #-16]!",
    "   stp x3, x4, [sp, #-16]!",
    "   stp x1, x2, [sp, #-16]!",
    "   bl _save_el_regs",
    "   ret",
    
    "_save_el_regs:",
    "   mrs x11, currentel",
    "   cmp x11, #0xc    // Check if EL3",
    "   b.eq 1f",
    "   cmp x11, #0x8    // Check if EL2",
    "   b.eq 2f",
    "   cmp x11, #0x4    // Check if EL1",
    "   b.eq 3f",
    "1: // EL3",
    "   mrs x1, esr_el3",
    "   mrs x2, elr_el3",
    "   b 4f",
    "2: // EL2",
    "   mrs x1, esr_el2",
    "   mrs x2, elr_el2",
    "   b 4f",
    "3: // EL1",
    "   mrs x1, esr_el1",
    "   mrs x2, elr_el1",
    "4: // Common",
    "   stp x2, x0, [sp, #-16]!",
    "   mov x0, sp",
    "   ret",
    
    "exception_exit:",
    "   ldp x2, x0, [sp], #16",
    "   mrs x11, currentel",
    "   cmp x11, #0xc    // Check if EL3",
    "   b.eq 1f",
    "   cmp x11, #0x8    // Check if EL2",
    "   b.eq 2f",
    "   cmp x11, #0x4    // Check if EL1",
    "   b.eq 3f",
    "1: // EL3",
    "   msr elr_el3, x2",
    "   b _restore_regs",
    "2: // EL2",
    "   msr elr_el2, x2",
    "   b _restore_regs",
    "3: // EL1",
    "   msr elr_el1, x2",
    
    "_restore_regs:",
    "   ldp x1, x2, [sp], #16",
    "   ldp x3, x4, [sp], #16",
    "   ldp x5, x6, [sp], #16",
    "   ldp x7, x8, [sp], #16",
    "   ldp x9, x10, [sp], #16",
    "   ldp x11, x12, [sp], #16",
    "   ldp x13, x14, [sp], #16",
    "   ldp x15, x16, [sp], #16",
    "   ldp x17, x18, [sp], #16",
    "   ldp x19, x20, [sp], #16",
    "   ldp x21, x22, [sp], #16",
    "   ldp x23, x24, [sp], #16",
    "   ldp x25, x26, [sp], #16",
    "   ldp x27, x28, [sp], #16",
    "   ldp x29, x30, [sp], #16",
    "   eret",
);

#[no_mangle]
extern "C" fn do_sync_handler() {
    uart::puts("Synchronous Exception\r\n");
}

#[no_mangle]
extern "C" fn do_irq_handler() {
    uart::puts("IRQ Exception\r\n");
    
    // Get the interrupt ID
    let irq_id = gic::get_interrupt_id();
    
    // Handle the interrupt
    if irq_id < 1023 {
        uart::puts("IRQ ID: ");
        print_hex(irq_id as u64);
        uart::puts("\r\n");
        
        // End the interrupt
        gic::end_of_interrupt(irq_id);
    }
}

#[no_mangle]
extern "C" fn do_fiq_handler() {
    uart::puts("FIQ Exception\r\n");
}

#[no_mangle]
extern "C" fn do_error_handler() {
    uart::puts("SError Exception\r\n");
}

// Helper function to print hex values using direct UART access
fn print_hex(value: u64) {
    const HEX_CHARS: [u8; 16] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', 
                                b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F'];
    
    uart::puts("0x");
    
    for i in (0..16).rev() {
        let digit = ((value >> (i * 4)) & 0xF) as usize;
        uart::putc(HEX_CHARS[digit]);
    }
}

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

// Vector base address (defined in assembly)
extern "C" {
    static exception_vector_table: u64;
}