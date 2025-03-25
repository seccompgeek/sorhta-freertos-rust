use core::arch::asm;

// Exception levels
pub const EL0: u8 = 0;
pub const EL1: u8 = 1;
pub const EL2: u8 = 2;
pub const EL3: u8 = 3;

// Cache operations
pub unsafe fn invalidate_icache_all() {
    asm!("ic ialluis");
    asm!("dsb ish");
    asm!("isb");
}

pub unsafe fn invalidate_dcache_all() {
    // TODO: Implement proper D-cache invalidation
    // This is a simplified placeholder
    asm!("dsb sy");
}

// Enable IRQ interrupts
pub unsafe fn enable_irq() {
    // Enable interrupts using MSR instruction directly
    asm!("msr daifclr, #2");
}

// Disable IRQ interrupts
pub unsafe fn disable_irq() {
    // Disable interrupts using MSR instruction directly
    asm!("msr daifset, #2");
}

// Enable FIQ interrupts
pub unsafe fn enable_fiq() {
    asm!("msr daifclr, #1");
}

// Disable FIQ interrupts
pub unsafe fn disable_fiq() {
    asm!("msr daifset, #1");
}

// Check if currently in IRQ context
pub fn is_in_irq() -> bool {
    let spsr: u64;
    unsafe {
        asm!("mrs {}, spsr_el1", out(reg) spsr);
    }
    // Check M[3:0] in SPSR for IRQ mode
    (spsr & 0xF) == 0x2
}

// Get the current exception level
pub fn current_el() -> u8 {
    let el: u64;
    unsafe {
        asm!("mrs {}, CurrentEL", out(reg) el);
    }
    ((el >> 2) & 0x3) as u8
}

// Get the CPU ID
pub fn cpu_id() -> u8 {
    let mut mpidr: u64;
    unsafe {
        asm!("mrs {}, mpidr_el1", out(reg) mpidr);
    }
    ((mpidr >> 8) & 0xFF) as u8
}

// Wait for event
pub fn wfe() {
    unsafe { asm!("wfe"); }
}

// Wait for interrupt
pub fn wfi() {
    unsafe { asm!("wfi"); }
}

// Data Synchronization Barrier
pub fn dsb() {
    unsafe { asm!("dsb sy"); }
}

// Data Memory Barrier
pub fn dmb() {
    unsafe { asm!("dmb sy"); }
}

// Instruction Synchronization Barrier
pub fn isb() {
    unsafe { asm!("isb"); }
}

// System register access helpers
pub unsafe fn write_sysreg(reg: &str, val: u64) {
    match reg {
        "vbar_el1" => asm!("msr vbar_el1, {}", in(reg) val),
        "ttbr0_el1" => asm!("msr ttbr0_el1, {}", in(reg) val),
        "tcr_el1" => asm!("msr tcr_el1, {}", in(reg) val),
        "mair_el1" => asm!("msr mair_el1, {}", in(reg) val),
        "sctlr_el1" => asm!("msr sctlr_el1, {}", in(reg) val),
        _ => panic!("Unsupported system register write"),
    }
}

pub unsafe fn read_sysreg(reg: &str) -> u64 {
    let val: u64;
    match reg {
        "vbar_el1" => asm!("mrs {}, vbar_el1", out(reg) val),
        "ttbr0_el1" => asm!("mrs {}, ttbr0_el1", out(reg) val),
        "tcr_el1" => asm!("mrs {}, tcr_el1", out(reg) val),
        "mair_el1" => asm!("mrs {}, mair_el1", out(reg) val),
        "sctlr_el1" => asm!("mrs {}, sctlr_el1", out(reg) val),
        _ => panic!("Unsupported system register read"),
    }
    val
}