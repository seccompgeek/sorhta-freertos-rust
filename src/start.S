.section .text.boot
.global _start

_start:
    // Identify which Exception Level we're in
    mrs     x0, CurrentEL
    and     x0, x0, #0xC        // Clear all bits except [3:2] which indicate EL

    // Compare the EL value and branch accordingly
    cmp     x0, #0x8            // EL2?
    beq     el2_to_el1
    cmp     x0, #0xC            // EL3?
    beq     el3_to_el1
    
    // If we're already in EL1, jump straight to main setup
    b       setup_el1

el3_to_el1:
    // Configure SCR_EL3 to set NS bit and route all exceptions to EL1
    mrs     x0, scr_el3
    orr     x0, x0, #(1 << 0)   // Set NS bit (Non-secure)
    orr     x0, x0, #(1 << 10)  // Set RW bit (EL2 is AArch64)
    msr     scr_el3, x0

    // Initialize SCTLR_EL1
    msr     sctlr_el1, xzr
    
    // Set up SPSR for exception return to EL1h
    mov     x0, #0x3c5          // EL1h (SPSel=1) with all interrupts masked
    msr     spsr_el3, x0
    
    // Set EL1 entry point and return
    adr     x0, setup_el1
    msr     elr_el3, x0
    eret

el2_to_el1:
    // Configure HCR_EL2
    mrs     x0, hcr_el2
    orr     x0, x0, #(1 << 31)  // Set RW bit (EL1 is AArch64)
    msr     hcr_el2, x0
    
    // Initialize SCTLR_EL1
    msr     sctlr_el1, xzr
    
    // Set EL1 stack pointer (we're not using it while in EL2, but set it anyway)
    adrp    x2, __stack_end
    add     x2, x2, :lo12:__stack_end
    msr     sp_el1, x2
    
    // Set up SPSR for exception return to EL1h
    mov     x0, #0x3c5          // EL1h (SPSel=1) with all interrupts masked
    msr     spsr_el2, x0
    
    // Set EL1 entry point and return
    adr     x0, setup_el1
    msr     elr_el2, x0
    eret

setup_el1:
    // We're now in EL1, set up the stack
    adrp    x2, __stack_end
    add     x2, x2, :lo12:__stack_end
    mov     sp, x2

    // Clear BSS section using ADRP
    adrp    x1, __bss_start
    add     x1, x1, :lo12:__bss_start
    adrp    x2, __bss_end
    add     x2, x2, :lo12:__bss_end
    cmp     x1, x2
    b.eq    2f

1:  str     xzr, [x1], #8 
    cmp     x1, x2
    b.lo    1b

2:
    // Initialize CPU
    bl      cpu_init

    // Jump to Rust code
    bl      kernel_init
    
    // Should never reach here, but if it does, loop forever
    b       halt

// Simple CPU initialization
cpu_init:
    // Disable MMU
    mrs     x0, sctlr_el1
    bic     x0, x0, #(1 << 0)   // Clear M bit
    msr     sctlr_el1, x0
    
    // Disable caches
    mrs     x0, sctlr_el1
    bic     x0, x0, #(1 << 2)   // Clear C bit (data cache)
    bic     x0, x0, #(1 << 12)  // Clear I bit (instruction cache)
    msr     sctlr_el1, x0
    
    // Invalidate caches
    ic      iallu                // Invalidate all instruction caches to PoU
    
    // Invalidate TLB
    tlbi    vmalle1              // Invalidate all TLB entries
    
    ret

halt:
    wfe                         // Wait for event (power efficient wait)
    b       halt                // Loop forever