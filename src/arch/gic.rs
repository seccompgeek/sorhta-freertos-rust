// S32G3 GIC-500 Interrupt Controller implementation
// Based on ARM GICv3 Architecture

use core::ptr::{read_volatile, write_volatile};
use core::arch::asm;
use alloc::format;

use crate::drivers::uart;

use crate::arch::s32g3::GIC_DIST_BASE;

// GIC Distributor register offsets
const GICD_CTLR: usize = 0x0000;           // Distributor Control Register
const GICD_TYPER: usize = 0x0004;          // Interrupt Controller Type Register
const GICD_IIDR: usize = 0x0008;           // Distributor Implementer Identification Register
const GICD_IGROUPR: usize = 0x0080;        // Interrupt Group Registers
const GICD_ISENABLER: usize = 0x0100;      // Interrupt Set-Enable Registers
const GICD_ICENABLER: usize = 0x0180;      // Interrupt Clear-Enable Registers
const GICD_ISPENDR: usize = 0x0200;        // Interrupt Set-Pending Registers
const GICD_ICPENDR: usize = 0x0280;        // Interrupt Clear-Pending Registers
const GICD_ISACTIVER: usize = 0x0300;      // Interrupt Set-Active Registers
const GICD_ICACTIVER: usize = 0x0380;      // Interrupt Clear-Active Registers
const GICD_IPRIORITYR: usize = 0x0400;     // Interrupt Priority Registers
const GICD_ITARGETSR: usize = 0x0800;      // Interrupt Processor Targets Registers
const GICD_ICFGR: usize = 0x0C00;          // Interrupt Configuration Registers
const GICD_IGRPMODR: usize = 0x0D00;       // Interrupt Group Modifier Registers
const GICD_NSACR: usize = 0x0E00;          // Non-secure Access Control Registers

// GIC Redistributor registers
const GICR_CTLR: usize = 0x00000;          // Redistributor Control Register
const GICR_TYPER: usize = 0x00008;         // Redistributor Type Register
const GICR_WAKER: usize = 0x00014;         // Redistributor Wake Register

// GIC register bit definitions
const GICD_CTLR_ENABLE: u32 = 0x1;
const GICD_CTLR_ARE_NS: u32 = 1 << 4;      // Affinity Routing Enable (Non-Secure)
const GICR_WAKER_PROCESSORASLEEP: u32 = 1 << 1;
const GICR_WAKER_CHILDRENASLEEP: u32 = 1 << 2;

// Number of interrupt IDs supported by the GIC
const GIC_MAX_INTID: u32 = 1020;
const GIC_MAX_SPI: u32 = 988;              // Shared Peripheral Interrupts: 32-1019
const GIC_MAX_PPI: u32 = 32;               // Private Peripheral Interrupts: 16-31
const GIC_MAX_SGI: u32 = 16;               // Software Generated Interrupts: 0-15

// GIC configuration constants
const GIC_PRIORITY_MASK: u32 = 0xF0;       // Priority mask (higher 4 bits)
const GIC_HIGHEST_PRIORITY: u32 = 0x0;     // Highest priority
const GIC_LOWEST_PRIORITY: u32 = 0xF0;     // Lowest priority
const GIC_DEFAULT_PRIORITY: u32 = 0xA0;    // Default priority


// GIC register addresses for S32G3
const GIC_DISTRIBUTOR_BASE: usize = 0x50800000;
const GIC_REDISTRIBUTOR_BASE: usize = 0x50880000;
const GIC_CPU_INTERFACE_BASE: usize = 0x50900000;

// GIC register offsets
const GICC_CTLR: usize = 0x000;
const GICC_PMR: usize = 0x004;
const GICC_IAR: usize = 0x00C;
const GICC_EOIR: usize = 0x010;

extern "C" {
    fn setup_vector_table();
}

// Memory barrier functions using cortex-a crate
fn dmb() {
    cortex_a::asm::barrier::dmb(cortex_a::asm::barrier::SY);
}

fn dsb() {
    cortex_a::asm::barrier::dsb(cortex_a::asm::barrier::SY);
}

fn isb() {
    cortex_a::asm::barrier::isb(cortex_a::asm::barrier::SY);
}

// Read 32-bit register
unsafe fn read_reg32(addr: usize) -> u32 {
    read_volatile(addr as *const u32)
}

// Write 32-bit register
unsafe fn write_reg32(addr: usize, val: u32) {
    write_volatile(addr as *mut u32, val);
}


/**
 * Get the number of SPIs supported by the GIC
 */
fn gic_num_spis() -> u32 {
    unsafe {
        let typer = read_volatile((GIC_DIST_BASE + GICD_TYPER) as *const u32);
        ((typer & 0x1F) + 1) * 32           // ITLinesNumber * 32
    }
}

/**
 * Initialize the GIC Distributor
 */
pub fn init_gicd() {
    unsafe {
        // Disable the distributor
        write_volatile((GIC_DIST_BASE + GICD_CTLR) as *mut u32, 0);
        
        // Get number of SPIs
        let num_spis = gic_num_spis();
        let num_ints = num_spis + GIC_MAX_PPI;
        
        // Calculate number of register sets needed (32 interrupts per word)
        let num_irq_regs = ((num_ints + 31) / 32) as usize;
        
        // Configure all SPIs as level-triggered, active high
        for i in 0..num_irq_regs {
            // SPIs start at ID 32
            if i >= 1 {
                write_volatile(((GIC_DIST_BASE + GICD_ICFGR) + (i * 4)) as *mut u32, 0);
            }
        }

        // Disable all interrupts
        for i in 0..num_irq_regs {
            write_volatile(((GIC_DIST_BASE + GICD_ICENABLER) + (i * 4)) as *mut u32, 0xFFFFFFFF);
        }

        // Clear any pending interrupts
        for i in 0..num_irq_regs {
            write_volatile(((GIC_DIST_BASE + GICD_ICPENDR) + (i * 4)) as *mut u32, 0xFFFFFFFF);
            write_volatile(((GIC_DIST_BASE + GICD_ICACTIVER) + (i * 4)) as *mut u32, 0xFFFFFFFF);
        }

        // Set priority for all interrupts
        let num_prio_regs = num_ints as usize;
        for i in 0..num_prio_regs {
            write_volatile(((GIC_DIST_BASE + GICD_IPRIORITYR) + (i * 4)) as *mut u32, GIC_DEFAULT_PRIORITY);
        }

        // Set interrupt targets to the primary core (legacy mode)
        let num_target_regs = num_ints as usize;
        for i in 32..num_target_regs {
            write_volatile(((GIC_DIST_BASE + GICD_ITARGETSR) + (i * 4)) as *mut u32, 0x01010101);
        }

        // Set all interrupts as Group 1 Non-secure
        for i in 0..num_irq_regs {
            write_volatile(((GIC_DIST_BASE + GICD_IGROUPR) + (i * 4)) as *mut u32, 0xFFFFFFFF);
        }

        // Enable the distributor with ARE_NS
        write_volatile((GIC_DIST_BASE + GICD_CTLR) as *mut u32, GICD_CTLR_ENABLE | GICD_CTLR_ARE_NS);
    }
}

/**
 * Initialize the GIC CPU Interface using system registers
 */
pub fn init_gicc() {
    unsafe {
        // Set priority mask to allow all interrupts
        asm!(
            "msr S3_0_C4_C6_0, {x:x}",
            x = in(reg) 0xFF_u64,
            options(nostack)
        );
        
        // Enable system register interface
        let mut sre: u64;
        asm!(
            "mrs {x}, S3_0_C12_C12_5",
            x = out(reg) sre,
            options(nostack)
        );
        sre |= 0x7;  // Enable, DFB, DIB bits
        asm!(
            "msr S3_0_C12_C12_5, {x}",
            x = in(reg) sre,
            options(nostack)
        );
        
        // Enable Group 1 interrupts
        asm!(
            "msr S3_0_C12_C12_7, {x:x}",
            x = in(reg) 0x1_u64,
            options(nostack)
        );
    }
}

/**
 * Initialize GIC Redistributor for this core
 */
pub fn init_gicr(core_id: u32) {
    unsafe {
        // Calculate base address for this core's redistributor
        // S32G3 redistributor stride is 0x20000
        let gicr_base = 0x50880000 + (core_id as usize * 0x20000);
        
        // Wake up the redistributor
        let waker = read_volatile((gicr_base + GICR_WAKER) as *const u32);
        write_volatile((gicr_base + GICR_WAKER) as *mut u32, waker & !GICR_WAKER_PROCESSORASLEEP);
        
        // Wait until redistributor is no longer asleep
        while (read_volatile((gicr_base + GICR_WAKER) as *const u32) & GICR_WAKER_CHILDRENASLEEP) != 0 {
            // Spin
        }
    }
}

/**
 * Initialize the GIC for this core
 */
pub fn init() {
    // Get current core ID
    let cpu_id = crate::arch::cpu_id() as u32;
    
    // Initialize GIC components
    if cpu_id == 0 {
        // Core 0 initializes the distributor
        init_gicd();
    }
    
    // Each core initializes its own redistributor and CPU interface
    init_gicr(cpu_id);
    init_gicc();
}

/**
 * Enable a specific interrupt
 */
// Enable a specific interrupt
pub fn enable_interrupt(id: u32) {
    unsafe {
        let reg_offset = GICD_ISENABLER + ((id as usize/ 32) * 4);
        let bit = 1 << (id % 32);
        
        write_reg32(GIC_DISTRIBUTOR_BASE + reg_offset, bit);
        dsb();
    }
}

/**
 * Disable a specific interrupt
 */
pub fn disable_interrupt(irq_num: u32) {
    unsafe {
        let reg_offset = (irq_num / 32) as usize;
        let bit_offset = irq_num % 32;
        
        write_volatile(
            ((GIC_DIST_BASE + GICD_ICENABLER) + (reg_offset * 4)) as *mut u32,
            1 << bit_offset
        );
    }
}

/**
 * Get the current interrupt ID (acknowledges the interrupt)
 */
pub fn get_interrupt_id() -> u32 {
    let iar: u64;
    unsafe {
        asm!(
            "mrs {x}, S3_0_C12_C12_0",
            x = out(reg) iar,
            options(nostack)
        );
    }
    (iar & 0x3FF) as u32
}

/**
 * Signal End Of Interrupt
 */
pub fn end_of_interrupt(irq_num: u32) {
    unsafe {
        asm!(
            "msr S3_0_C12_C12_1, {x}",
            x = in(reg) irq_num as u64,
            options(nostack)
        );
    }
}

/**
 * Set interrupt priority
 */
pub fn set_priority(irq_num: u32, priority: u8) {
    unsafe {
        let reg_offset = irq_num as usize;
        let priority_val = (priority as u32) << 4; // Higher 4 bits are used
        
        write_volatile(
            ((GIC_DIST_BASE + GICD_IPRIORITYR) + (reg_offset * 4)) as *mut u32,
            priority_val
        );
    }
}

/**
 * Send a Software Generated Interrupt
 */
pub fn send_sgi(sgi_id: u32, target_list: u8, _filter: u8) {
    if sgi_id > 15 {
        return; // Invalid SGI ID
    }
    
    unsafe {
        // In GICv3, SGIs are sent using system registers
        let sgi_value = (sgi_id as u64) | ((target_list as u64) << 16);
        asm!(
            "msr S3_0_C12_C11_5, {x}",
            x = in(reg) sgi_value,
            options(nostack)
        );
    }
}

// Acknowledge an interrupt and get its ID
pub fn acknowledge_interrupt() -> u32 {
    unsafe {
        let int_id = read_volatile((GIC_CPU_INTERFACE_BASE + GICC_IAR) as *mut u32) & 0x3FF;
        dmb();
        int_id
    }
}

// Main IRQ handler called from exceptions.rs
pub fn handle_irq() {
    let int_id = acknowledge_interrupt();
    
    if int_id >= 1022 {
        // Spurious interrupt
        return;
    }
    
    uart::puts(&format!("Handling interrupt {}\n", int_id));
    
    // Handle specific interrupts
    match int_id {
        32 => {
            uart::puts("Timer interrupt\n");
            // Handle timer interrupt
        },
        // Add other interrupt handlers as needed
        _ => {
            uart::puts(&format!("Unhandled interrupt {}\n", int_id));
        }
    }
    
    end_interrupt(int_id);
}

// Complete handling of an interrupt
pub fn end_interrupt(id: u32) {
    unsafe {
        dmb();
        write_reg32(GIC_CPU_INTERFACE_BASE + GICC_EOIR, id);
        dsb();
    }
}