.section .text.boot
.global _start

// Memory Attribute Indirection Register (MAIR) values
.equ MAIR_DEVICE_nGnRnE, 0x00    // Device, non-Gathering, non-Reordering, no Early write acknowledgement
.equ MAIR_DEVICE_nGnRE,  0x04    // Device, non-Gathering, non-Reordering, Early write acknowledgement
.equ MAIR_NORMAL_NC,     0x44    // Normal, Outer/Inner Non-Cacheable
.equ MAIR_NORMAL_WT,     0xBB    // Normal, Outer/Inner Write-Through Cacheable
.equ MAIR_NORMAL_WB,     0xFF    // Normal, Outer/Inner Write-Back Cacheable

// Memory region size definitions (for sections)
.equ SIZE_1K,   0x0400
.equ SIZE_2K,   0x0800
.equ SIZE_4K,   0x1000
.equ SIZE_16K,  0x4000
.equ SIZE_64K,  0x10000
.equ SIZE_1M,   0x100000
.equ SIZE_2M,   0x200000
.equ SIZE_1G,   0x40000000

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

    // Setup Vector table 
    bl      setup_vector_table

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
    // Save return address
    mov     x28, x30

    // Disable MMU and caches first
    mrs     x0, sctlr_el1
    bic     x0, x0, #(1 << 0)   // Clear M bit (MMU)
    bic     x0, x0, #(1 << 2)   // Clear C bit (data cache)
    bic     x0, x0, #(1 << 12)  // Clear I bit (instruction cache)
    msr     sctlr_el1, x0
    isb

    // Invalidate instruction cache and TLB
    ic      iallu                // Invalidate all instruction caches to PoU
    tlbi    vmalle1              // Invalidate all TLB entries
    dsb     ish                  // Ensure completion of cache operation
    isb                          // Synchronize context

    // Invalidate data cache - Cortex-A53 specific
    bl      invalidate_dcache_all

    // Enable SCU (S32G3 specific) - BEFORE MMU enable
    ldr     x0, =0x30600000
    ldr     w1, [x0]
    orr     w1, w1, #0x1
    str     w1, [x0]
    dsb     sy

    // Configure MMU and memory attributes
    bl      setup_mmu

    // Cortex-A53 specific setup - CPU Auxiliary Control Register
    mrs     x0, S3_1_c15_c2_1   // Read CPUACTLR_EL1
    orr     x0, x0, #(1 << 0)   // Enable data prefetch
    orr     x0, x0, #(1 << 1)   // Enable instruction prefetch
    msr     S3_1_c15_c2_1, x0   // Write CPUACTLR_EL1

    // L2 cache already configured by boot core
    // We only need to ensure our core can use it correctly
    // L2 Cache configuration (if not already done by boot core)
    // Read current L2 Control Register value
    mrs     x0, S3_1_C11_C0_2   // S3_1_C11_C0_2 is the L2CTLR
    orr     x0, x0, #(1 << 26)  // Enable L2 data prefetch
    msr     S3_1_C11_C0_2, x0

    // Configure SCTLR_EL1 to enable MMU and caches
    mrs     x0, sctlr_el1

    // Standard cache and MMU bits
    orr     x0, x0, #(1 << 0)    // Enable MMU (M bit)
    orr     x0, x0, #(1 << 2)    // Enable data cache (C bit)
    orr     x0, x0, #(1 << 12)   // Enable instruction cache (I bit)
    orr     x0, x0, #(1 << 11)   // Enable branch prediction (Z bit)

    // Additional reliability/performance features
    orr     x0, x0, #(1 << 5)    // Enable CP15 barrier operations (CP15BEN)
    orr     x0, x0, #(1 << 14)   // Instruction cache coherence (DZE bit)
    orr     x0, x0, #(1 << 3)    // Enable SP alignment check (SA bit)
    orr     x0, x0, #(1 << 4)    // Enable alignment checking (A bit)

    // Apply the configuration
    msr     sctlr_el1, x0
    isb                          // Synchronize context

    // Return to caller
    mov     x30, x28
    ret


// Setup MMU translation tables and registers
setup_mmu:
    // Save LR
    mov     x29, x30

    // 1. Clear TTBR0
    adrp    x0, ttb0_base
    add     x0, x0, :lo12:ttb0_base
    mov     x1, #0x4000      // 16KB table
    bl      memzero

    // 2. Map DDR (S32G3 specific)
    adrp    x0, ttb0_base
    add     x0, x0, :lo12:ttb0_base
    mov     x1, #0xE0000000   // DDR start
    mov     x2, #0xE2000000   // 32MB DDR
    mov     x3, x1            // Identity mapping
    mov     x4, #2            // WB cacheable
    mov     x5, #SIZE_2M      // Use 2MB blocks instead of 1GB
    bl      create_block_mappings

    // Shared memory
    mov     x1, #0xE2000000         // Shared memory start
    add     x2, x1, #0x100000       // End address (start + 1MB)
    mov     x3, x1                  // Identity mapping
    mov     x4, #0                  // Device memory (non-cacheable) attribute index
    mov     x5, #SIZE_2M            // Use 2MB blocks
    bl      create_block_mappings

    // 3. Map peripherals
    mov     x1, #0x40000000   // Peripherals start
    mov     x2, #0x50000000   // 256MB
    mov     x3, x1
    mov     x4, #0            // Device-nGnRnE
    mov     x5, #SIZE_2M
    bl      create_block_mappings

    // 4. Configure MAIR
    // 4. Configure MAIR
    mov     x0, #0x44                // Index 0: Device (0x44)
    movk    x0, #0x00, lsl #16       // Index 1: Unused (0x00)
    movk    x0, #0x00, lsl #32       // Index 2: Unused (0x00)
    movk    x0, #0xBB, lsl #48       // Index 3: Normal WB (0xBB)
    msr     mair_el1, x0

    // 5. Configure TCR_EL1 (48-bit VA)
    mov x0, #(64-48)<<0 | (0b01<<8) | (0b01<<10) // T0SZ=16, IRGN=WBWA
    orr x0, x0, #(0b10<<14)  // TG0=4KB
    orr x0, x0, #(0b11<<12)  // SH=Inner
    msr tcr_el1, x0

    // 6. Finalize
    msr ttbr0_el1, x0
    dsb sy
    isb

    ret

// Create block mappings in the translation table
// x0 = table base address
// x1 = start physical address
// x2 = end physical address
// x3 = start VA (usually same as physical for identity mapping)
// x4 = memory attribute index
// x5 = block size (e.g., 2MB)
create_block_mappings:
    // Save link register and callee-saved registers
    stp     x29, x30, [sp, #-16]!
    stp     x19, x20, [sp, #-16]!
    stp     x21, x22, [sp, #-16]!
    stp     x23, x24, [sp, #-16]!

    // Setup local variables
    mov     x19, x0              // Table base
    mov     x20, x1              // Current physical address
    mov     x21, x2              // End physical address
    mov     x22, x3              // Current VA
    mov     x23, x4              // Attribute index
    mov     x24, x5              // Block size (passed in as parameter)

1:  // Check if we're done
    cmp     x20, x21
    b.hs    2f

    // Calculate table entry address based on block size
    movz    x8, #0x4000, lsl #16  // Lower half of 0x40000000 (1GB)
    movk    x8, #0x0000, lsl #0   // Upper half of 0x40000000
    cmp     x24, x8               // Check if 1GB blocks
    b.eq    f1_l1_entry
    
    // For 2MB blocks (level 2 entry)
    lsr     x7, x22, #21         // Shift VA by 21 bits (for 2MB blocks)
    and     x7, x7, #0x1FF       // Extract level 2 index bits (9 bits)
    b       f1_create_entry
    
f1_l1_entry:
    // For 1GB blocks (level 1 entry)
    lsr     x7, x22, #30         // Shift VA by 30 bits (for 1GB blocks)
    and     x7, x7, #0x1FF       // Extract level 1 index bits (9 bits)
    
f1_create_entry:
    add     x6, x19, x7, lsl #3  // Each entry is 8 bytes

    // Create block descriptor
    mov     x9, #0x1             // Block descriptor (as opposed to table)
    orr     x9, x9, x20          // Add physical address

    // Add attributes based on memory type
    lsl     x10, x23, #2         // Attribute index * 4 (shift to position)
    orr     x9, x9, x10          // Apply attribute index

    // Add access flags
    orr     x9, x9, #(1 << 10)   // Access flag
    // Set as non-global, accessible from EL1
    orr     x9, x9, #(1 << 11)   // Non-global bit
    orr     x9, x9, #(0b01 << 6) // EL1 Read/Write, EL0 no access

    // Store the entry
    str     x9, [x6]

    // Advance to next block
    add     x20, x20, x24
    add     x22, x22, x24
    b       1b

2:  // Restore registers and return
    ldp     x23, x24, [sp], #16
    ldp     x21, x22, [sp], #16
    ldp     x19, x20, [sp], #16
    ldp     x29, x30, [sp], #16
    ret


// Invalidate entire data cache
invalidate_dcache_all:
    // Read CSSELR for cache size info
    mrs     x0, clidr_el1
    and     x3, x0, #0x7000000   // Extract Level of Coherence
    lsr     x3, x3, #23          // Right shift to get LoC
    cbz     x3, 5f               // If 0, no need to clean
    
    mov     x10, #0              // Start from cache level 0
    
1:  // For each cache level
    add     x2, x10, x10, lsl #1  // x2 = level * 3
    lsr     x1, x0, x2           // x1 = cache type for this level
    and     x1, x1, #7           // Extract cache type
    cmp     x1, #2               // Check if no cache or only instruction cache
    b.lt    4f                   // Skip if so
    
    // Select this cache level
    msr     csselr_el1, x10      // Select current cache level
    isb                          // Synchronize change
    
    // Get cache size ID register for this level
    mrs     x1, ccsidr_el1
    
    // Extract set and way info
    and     x2, x1, #7           // Line size encoding
    add     x2, x2, #4           // Line size = 2^(x+4) words
    mov     x4, #16              // Convert to bytes
    lsl     x4, x4, x2           // x4 = cache line size in bytes
    
    // Number of ways
    mov     x5, #1
    and     x7, x1, #0x3800
    lsr     x7, x7, #11
    orr     x2, xzr, x7
    lsl     x5, x5, x2           // x5 = number of ways
    
    // Number of sets
    mov     x6, #1
    and     x2, x1, #0x7FFF80
    lsr     x2, x2, #7
    orr     x2, xzr, x2
    lsl     x6, x6, x2           // x6 = number of sets
    
    // For each set/way combination
2:  // For each set
    mov     x8, #0               // Way counter
    
3:  // For each way
    lsl     x9, x8, x2           // Set way number
    orr     x11, x10, x9         // Add cache level
    lsl     x7, x7, #1           
    orr     x11, x11, x7         // Add in the way and set
    dc      cisw, x11           // Clean & invalidate by set/way
    add     x8, x8, #1           // Increment way counter
    cmp     x8, x5               // Check if done with ways
    b.lt    3b
    
    add     x7, x7, #1           // Increment set counter
    cmp     x7, x6               // Check if done with sets
    b.lt    2b
    
4:  // Next cache level
    add     x10, x10, #1
    cmp     x3, x10
    b.gt    1b
    
5:  // Ensure completion and return
    dsb     sy
    isb
    ret

// Zero memory
// x0 = start address
// x1 = size in bytes (must be multiple of 16)
memzero:
    // Check for invalid size
    cbz     x1, 2f
    
    // Calculate end address
    add     x2, x0, x1
    
1:  // Zero 16 bytes at a time
    stp     xzr, xzr, [x0], #16
    cmp     x0, x2
    b.lo    1b
    
2:  // Return
    ret

halt:
    wfe                         // Wait for event (power efficient wait)
    b       halt                // Loop forever