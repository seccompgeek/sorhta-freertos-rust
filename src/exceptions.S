.section ".text.exceptions"
.global exception_vector_table
.global setup_vector_table

// AArch64 exception vector table must be aligned to 2KB (2^11)
.align 11
exception_vector_table:
    // Current EL with SP0
    .align 7 // Each vector entry must be aligned to 128 bytes
    b       el1_sync_sp0
    .align 7
    b       el1_irq_sp0
    .align 7
    b       el1_fiq_sp0
    .align 7
    b       el1_serror_sp0

    // Current EL with SPx
    .align 7
    b       el1_sync_spx
    .align 7
    b       el1_irq_spx
    .align 7
    b       el1_fiq_spx
    .align 7
    b       el1_serror_spx

    // Lower EL using AArch64
    .align 7
    b       el0_sync_a64
    .align 7
    b       el0_irq_a64
    .align 7
    b       el0_fiq_a64
    .align 7
    b       el0_serror_a64

    // Lower EL using AArch32
    .align 7
    b       el0_sync_a32
    .align 7
    b       el0_irq_a32
    .align 7
    b       el0_fiq_a32
    .align 7
    b       el0_serror_a32

setup_vector_table:
    adr     x0, exception_vector_table
    msr     vbar_el1, x0
    isb
    ret

//----------------------------------------------------------------------
// Exception handlers
//----------------------------------------------------------------------

// Save all registers and call the Rust handler
.macro SAVE_REGISTERS
    sub     sp, sp, #(8 * 30)
    stp     x0, x1, [sp, #(8 * 0)]
    stp     x2, x3, [sp, #(8 * 2)]
    stp     x4, x5, [sp, #(8 * 4)]
    stp     x6, x7, [sp, #(8 * 6)]
    stp     x8, x9, [sp, #(8 * 8)]
    stp     x10, x11, [sp, #(8 * 10)]
    stp     x12, x13, [sp, #(8 * 12)]
    stp     x14, x15, [sp, #(8 * 14)]
    stp     x16, x17, [sp, #(8 * 16)]
    stp     x18, x19, [sp, #(8 * 18)]
    stp     x20, x21, [sp, #(8 * 20)]
    stp     x22, x23, [sp, #(8 * 22)]
    stp     x24, x25, [sp, #(8 * 24)]
    stp     x26, x27, [sp, #(8 * 26)]
    stp     x28, x29, [sp, #(8 * 28)]
.endm

// Restore registers and return
.macro RESTORE_REGISTERS
    ldp     x0, x1, [sp, #(8 * 0)]
    ldp     x2, x3, [sp, #(8 * 2)]
    ldp     x4, x5, [sp, #(8 * 4)]
    ldp     x6, x7, [sp, #(8 * 6)]
    ldp     x8, x9, [sp, #(8 * 8)]
    ldp     x10, x11, [sp, #(8 * 10)]
    ldp     x12, x13, [sp, #(8 * 12)]
    ldp     x14, x15, [sp, #(8 * 14)]
    ldp     x16, x17, [sp, #(8 * 16)]
    ldp     x18, x19, [sp, #(8 * 18)]
    ldp     x20, x21, [sp, #(8 * 20)]
    ldp     x22, x23, [sp, #(8 * 22)]
    ldp     x24, x25, [sp, #(8 * 24)]
    ldp     x26, x27, [sp, #(8 * 26)]
    ldp     x28, x29, [sp, #(8 * 28)]
    add     sp, sp, #(8 * 30)
.endm

el1_sync_sp0:
    SAVE_REGISTERS
    bl      handle_el1_sync_exception
    RESTORE_REGISTERS
    eret

el1_irq_sp0:
    SAVE_REGISTERS
    bl      handle_el1_irq
    RESTORE_REGISTERS
    eret

el1_fiq_sp0:
    SAVE_REGISTERS
    bl      handle_el1_fiq
    RESTORE_REGISTERS
    eret

el1_serror_sp0:
    SAVE_REGISTERS
    bl      handle_el1_serror
    RESTORE_REGISTERS
    eret

el1_sync_spx:
    SAVE_REGISTERS
    bl      handle_el1_sync_exception
    RESTORE_REGISTERS
    eret

el1_irq_spx:
    SAVE_REGISTERS
    bl      handle_el1_irq
    RESTORE_REGISTERS
    eret

el1_fiq_spx:
    SAVE_REGISTERS
    bl      handle_el1_fiq
    RESTORE_REGISTERS
    eret

el1_serror_spx:
    SAVE_REGISTERS
    bl      handle_el1_serror
    RESTORE_REGISTERS
    eret

el0_sync_a64:
    SAVE_REGISTERS
    mrs     x0, esr_el1
    mrs     x1, far_el1
    bl      handle_el0_sync
    RESTORE_REGISTERS
    eret

el0_irq_a64:
    SAVE_REGISTERS
    bl      handle_el0_irq
    RESTORE_REGISTERS
    eret

el0_fiq_a64:
    SAVE_REGISTERS
    bl      handle_el0_fiq
    RESTORE_REGISTERS
    eret

el0_serror_a64:
    SAVE_REGISTERS
    bl      handle_el0_serror
    RESTORE_REGISTERS
    eret

el0_sync_a32:
    SAVE_REGISTERS
    bl      handle_el0_sync_a32
    RESTORE_REGISTERS
    eret

el0_irq_a32:
    SAVE_REGISTERS
    bl      handle_el0_irq_a32
    RESTORE_REGISTERS
    eret

el0_fiq_a32:
    SAVE_REGISTERS
    bl      handle_el0_fiq_a32
    RESTORE_REGISTERS
    eret

el0_serror_a32:
    SAVE_REGISTERS
    bl      handle_el0_serror_a32
    RESTORE_REGISTERS
    eret