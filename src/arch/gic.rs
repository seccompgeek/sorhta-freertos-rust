// gic.rs
// Generic Interrupt Controller driver for S32G3 secondary cores

use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

/// GIC register addresses for S32G3
pub const GIC_DIST_BASE: u64 = 0x5080_0000;
pub const GIC_CPU_BASE: u64 = 0x5088_0000;

/// GIC Distributor register offsets
pub const GICD_CTLR: u64 = 0x0000;           // Distributor Control Register
pub const GICD_TYPER: u64 = 0x0004;          // Interrupt Controller Type Register
pub const GICD_IIDR: u64 = 0x0008;           // Distributor Implementer Identification Register
pub const GICD_IGROUPR: u64 = 0x0080;        // Interrupt Group Registers
pub const GICD_ISENABLER: u64 = 0x0100;      // Interrupt Set-Enable Registers
pub const GICD_ICENABLER: u64 = 0x0180;      // Interrupt Clear-Enable Registers
pub const GICD_ISPENDR: u64 = 0x0200;        // Interrupt Set-Pending Registers
pub const GICD_ICPENDR: u64 = 0x0280;        // Interrupt Clear-Pending Registers
pub const GICD_ISACTIVER: u64 = 0x0300;      // Interrupt Set-Active Registers
pub const GICD_ICACTIVER: u64 = 0x0380;      // Interrupt Clear-Active Registers
pub const GICD_IPRIORITYR: u64 = 0x0400;     // Interrupt Priority Registers
pub const GICD_ITARGETSR: u64 = 0x0800;      // Interrupt Processor Targets Registers
pub const GICD_ICFGR: u64 = 0x0C00;          // Interrupt Configuration Registers
pub const GICD_SGIR: u64 = 0x0F00;           // Software Generated Interrupt Register

/// GIC CPU Interface register offsets
pub const GICC_CTLR: u64 = 0x0000;           // CPU Interface Control Register
pub const GICC_PMR: u64 = 0x0004;            // Priority Mask Register
pub const GICC_BPR: u64 = 0x0008;            // Binary Point Register
pub const GICC_IAR: u64 = 0x000C;            // Interrupt Acknowledge Register
pub const GICC_EOIR: u64 = 0x0010;           // End of Interrupt Register
pub const GICC_RPR: u64 = 0x0014;            // Running Priority Register
pub const GICC_HPPIR: u64 = 0x0018;          // Highest Priority Pending Interrupt Register
pub const GICC_ABPR: u64 = 0x001C;           // Aliased Binary Point Register
pub const GICC_DIR: u64 = 0x1000;            // Deactivate Interrupt Register

/// Track if GIC has been initialized
static GIC_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Constants for MPIDR register processing
pub const MPIDR_AFFINITY_MASK: u64 = 0xff00ff_ffff;
pub const MPIDR_MT_MASK: u64 = 1 << 24;
pub const MPIDR_AFFLVL_MASK: u64 = 0xff;
pub const MPIDR_AFFINITY_BITS: u32 = 8;
pub const MPIDR_CPU_MASK: u64 = MPIDR_AFFLVL_MASK;
pub const MPIDR_CLUSTER_MASK: u64 = MPIDR_AFFLVL_MASK << MPIDR_AFFINITY_BITS;
pub const MPIDR_AFF1_SHIFT: u32 = MPIDR_AFFINITY_BITS;
pub const S32_MPIDR_CPU_MASK_BITS: u32 = 3; // Log2 of max CPUs per cluster (8)

/// Platform-specific constants
pub const PLATFORM_CLUSTER_COUNT: u32 = 4;
pub const PLATFORM_MAX_CPUS_PER_CLUSTER: u32 = 8;

/// GIC Driver for S32G3 secondary cores
pub struct GicDriver;

impl GicDriver {
    /// Get the CPU's affinity value at the specified level
    #[inline(always)]
    fn mpidr_afflvl_val(mpidr: u64, level: u32) -> u32 {
        ((mpidr >> (level * MPIDR_AFFINITY_BITS)) & MPIDR_AFFLVL_MASK) as u32
    }
    
    /// Convert MPIDR to core position (assembly equivalent)
    #[inline(always)]
    fn s32_core_pos_by_mpidr(mpidr: u64) -> u32 {
        let cpu_id = mpidr & MPIDR_CPU_MASK;
        let cluster_id = (mpidr & MPIDR_CLUSTER_MASK) >> MPIDR_AFF1_SHIFT;
        
        (cpu_id + (cluster_id << S32_MPIDR_CPU_MASK_BITS)) as u32
    }
    
    /// Get platform core position by MPIDR with validation
    pub fn plat_core_pos_by_mpidr(mpidr: u64) -> Result<u32, &'static str> {
        // Mask out any bits that aren't part of the affinity fields
        let mpidr = mpidr & MPIDR_AFFINITY_MASK;
        
        // Check if any unexpected bits are set in the MPIDR value
        if mpidr & !(MPIDR_CLUSTER_MASK | MPIDR_CPU_MASK) != 0 {
            return Err("Invalid MPIDR value: unexpected bits set");
        }
        
        // Extract the cluster ID (Affinity Level 1) and CPU ID (Affinity Level 0)
        let cluster_id = Self::mpidr_afflvl_val(mpidr, 1);
        let cpu_id = Self::mpidr_afflvl_val(mpidr, 0);
        
        // Validate that the cluster ID and CPU ID are within platform limits
        if cluster_id >= PLATFORM_CLUSTER_COUNT || 
           cpu_id >= PLATFORM_MAX_CPUS_PER_CLUSTER {
            return Err("Invalid MPIDR value: cluster or CPU ID out of range");
        }
        
        // If all checks pass, calculate the core position
        Ok(Self::s32_core_pos_by_mpidr(mpidr))
    }
    
    /// Get the current CPU's core position
    pub fn plat_my_core_pos() -> Result<u32, &'static str> {
        let mpidr: u64;
        unsafe {
            asm!(
                "mrs {0}, mpidr_el1",
                out(reg) mpidr
            );
        }
        
        Self::plat_core_pos_by_mpidr(mpidr)
    }
    
    /// Get the current CPU ID from MPIDR_EL1 register
    /// This is a simplified version that just returns the core's Affinity Level 0
    #[inline(always)]
    pub fn get_cpu_id() -> u32 {
        match Self::plat_my_core_pos() {
            Ok(core_pos) => core_pos,
            Err(_) => {
                // Fallback to basic method if the robust method fails
                let mpidr: u64;
                unsafe {
                    asm!(
                        "mrs {0}, mpidr_el1",
                        "and {0}, {0}, #0xFF",
                        out(reg) mpidr
                    );
                }
                mpidr as u32
            }
        }
    }

    /// Initialize GIC for a secondary core
    pub fn init_secondary_core() {
        // Check if already initialized
        if GIC_INITIALIZED.load(Ordering::Relaxed) {
            return;
        }
        
        // Get core position using the robust method
        let core_pos = match Self::plat_my_core_pos() {
            Ok(pos) => pos,
            Err(_) => {
                // Fallback to simple method if the robust method fails
                Self::get_cpu_id()
            }
        };
        
        // Calculate CPU interface target mask (bit position in the target register)
        // This identifies which CPU within the cluster this core corresponds to
        let cpu_id = core_pos & ((1 << S32_MPIDR_CPU_MASK_BITS) - 1);
        
        unsafe {
            // 1. Initialize CPU interface
            
            // Set priority mask to allow all interrupts
            write_volatile((GIC_CPU_BASE + GICC_PMR) as *mut u32, 0xFF);
            
            // Set Binary Point Register for group priority
            write_volatile((GIC_CPU_BASE + GICC_BPR) as *mut u32, 0x7);
            
            // Enable CPU interface - enable both Group 0 and Group 1 interrupts
            write_volatile((GIC_CPU_BASE + GICC_CTLR) as *mut u32, 0x7);
            
            // 2. Enable this core's SGIs (Software Generated Interrupts)
            // SGIs are typically IDs 0-15
            // The ITARGETSR registers map 4 interrupt IDs per register, with 8 bits per ID
            // Each byte in the register represents the target list for one interrupt
            
            for sgi_id in 0..16 {
                // Calculate register offset: each register handles 4 interrupts
                let reg_offset = (sgi_id / 4) * 4;
                // Calculate byte position within the register (0, 1, 2, or 3)
                let byte_offset = sgi_id % 4;
                
                // Get the address of the appropriate ITARGETSR register
                let reg_addr = GIC_DIST_BASE + GICD_ITARGETSR + reg_offset as u64;
                
                // Read current targets for this register
                let mut targets = read_volatile(reg_addr as *const u32);
                
                // Set the bit corresponding to this CPU for the specific SGI
                // Each byte position has 8 bits for up to 8 CPUs
                let byte_shift = byte_offset * 8;
                let cpu_mask = (1u32 << cpu_id) << byte_shift;
                
                // Update the target register
                targets |= cpu_mask;
                write_volatile(reg_addr as *mut u32, targets);
            }
            
            // Mark as initialized
            GIC_INITIALIZED.store(true, Ordering::Relaxed);
        }
    }

    /// Enable interrupts for the secondary core
    #[inline(always)]
    pub fn enable_interrupts() {
        unsafe {
            // Clear DAIF bits to enable all interrupts
            asm!(
                "msr DAIFClr, #0xF",
                "isb"
            );
        }
    }

    /// Disable interrupts
    #[inline(always)]
    pub fn disable_interrupts() {
        unsafe {
            // Set DAIF bits to disable all interrupts
            asm!(
                "msr DAIFSet, #0xF",
                "isb"
            );
        }
    }

    /// Acknowledge an interrupt
    #[inline(always)]
    pub fn acknowledge_interrupt() -> u32 {
        unsafe {
            read_volatile((GIC_CPU_BASE + GICC_IAR) as *const u32)
        }
    }

    /// End an interrupt
    #[inline(always)]
    pub fn end_interrupt(interrupt_id: u32) {
        unsafe {
            write_volatile((GIC_CPU_BASE + GICC_EOIR) as *mut u32, interrupt_id);
        }
    }

    /// Generate a software interrupt to another core using core position
    pub fn send_sgi_to_core(target_core_pos: u32, sgi_id: u8) -> Result<(), &'static str> {
        if sgi_id > 15 {
            // SGIs are IDs 0-15
            return Err("Invalid SGI ID: must be 0-15");
        }
        
        // Extract the CPU ID from the core position
        // This gets the position within the cluster (0-7 typically)
        let target_cpu = target_core_pos & ((1 << S32_MPIDR_CPU_MASK_BITS) - 1);
        
        if target_cpu >= PLATFORM_MAX_CPUS_PER_CLUSTER {
            return Err("Invalid target CPU ID");
        }
        
        // Create target CPU mask (1 bit per target CPU)
        let target_cpu_mask = 1u8 << target_cpu;
        
        // Format SGIR value:
        // Bits 0-3: SGI ID
        // Bit 15: Use target list (1) vs. all but self (0)
        // Bits 16-23: Target list (one bit per CPU)
        let sgi_value = (((target_cpu_mask as u32) & 0xFF) << 16) | 
                        (1u32 << 15) |  // Use target list
                        ((sgi_id as u32) & 0xF);

        unsafe {
            write_volatile((GIC_DIST_BASE + GICD_SGIR) as *mut u32, sgi_value);
        }
        
        Ok(())
    }
    
    /// Generate a software interrupt to a set of cores using a CPU mask
    pub fn send_sgi(target_cpu_mask: u8, sgi_id: u8) -> Result<(), &'static str> {
        if sgi_id > 15 {
            // SGIs are IDs 0-15
            return Err("Invalid SGI ID: must be 0-15");
        }

        // Format SGIR value:
        // Bits 0-3: SGI ID
        // Bit 15: Use target list (1) vs. all but self (0)
        // Bits 16-23: Target list (one bit per CPU)
        let sgi_value = (((target_cpu_mask as u32) & 0xFF) << 16) | 
                        (1u32 << 15) |  // Use target list
                        ((sgi_id as u32) & 0xF);

        unsafe {
            write_volatile((GIC_DIST_BASE + GICD_SGIR) as *mut u32, sgi_value);
        }
        
        Ok(())
    }
    
    /// Enable a specific interrupt
    pub fn enable_interrupt(interrupt_id: u32) {
        if interrupt_id >= 1024 {
            return; // Invalid interrupt ID
        }
        
        unsafe {
            let reg_offset = (interrupt_id / 32) * 4;
            let bit_offset = interrupt_id % 32;
            let reg_addr = GIC_DIST_BASE + GICD_ISENABLER + reg_offset as u64;
            
            let value = 1u32 << bit_offset;
            write_volatile(reg_addr as *mut u32, value);
        }
    }
    
    /// Disable a specific interrupt
    pub fn disable_interrupt(interrupt_id: u32) {
        if interrupt_id >= 1024 {
            return; // Invalid interrupt ID
        }
        
        unsafe {
            let reg_offset = (interrupt_id / 32) * 4;
            let bit_offset = interrupt_id % 32;
            let reg_addr = GIC_DIST_BASE + GICD_ICENABLER + reg_offset as u64;
            
            let value = 1u32 << bit_offset;
            write_volatile(reg_addr as *mut u32, value);
        }
    }
    
    /// Set interrupt priority
    pub fn set_priority(interrupt_id: u32, priority: u8) {
        if interrupt_id >= 1024 {
            return; // Invalid interrupt ID
        }
        
        unsafe {
            let reg_addr = GIC_DIST_BASE + GICD_IPRIORITYR + interrupt_id as u64;
            write_volatile(reg_addr as *mut u8, priority);
        }
    }
}