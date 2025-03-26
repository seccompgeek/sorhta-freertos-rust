// gicv3.rs
// Generic Interrupt Controller v3 driver for S32G3 secondary cores

use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

/// GICv3 register addresses for S32G3
pub const GICD_BASE: u64 = 0x5080_0000;  // GIC Distributor base
pub const GICR_BASE: u64 = 0x5090_0000;  // GIC Redistributor base

/// GIC Distributor register offsets
pub const GICD_CTLR: u64 = 0x0000;           // Distributor Control Register
pub const GICD_TYPER: u64 = 0x0004;          // Interrupt Controller Type Register
pub const GICD_IIDR: u64 = 0x0008;           // Distributor Implementer Identification Register
pub const GICD_STATUSR: u64 = 0x0010;        // Error Reporting Status Register
pub const GICD_IGROUPR: u64 = 0x0080;        // Interrupt Group Registers
pub const GICD_ISENABLER: u64 = 0x0100;      // Interrupt Set-Enable Registers
pub const GICD_ICENABLER: u64 = 0x0180;      // Interrupt Clear-Enable Registers
pub const GICD_ISPENDR: u64 = 0x0200;        // Interrupt Set-Pending Registers
pub const GICD_ICPENDR: u64 = 0x0280;        // Interrupt Clear-Pending Registers
pub const GICD_ISACTIVER: u64 = 0x0300;      // Interrupt Set-Active Registers
pub const GICD_ICACTIVER: u64 = 0x0380;      // Interrupt Clear-Active Registers
pub const GICD_IPRIORITYR: u64 = 0x0400;     // Interrupt Priority Registers
pub const GICD_ICFGR: u64 = 0x0C00;          // Interrupt Configuration Registers
pub const GICD_IROUTER: u64 = 0x6000;        // Interrupt Routing Registers

/// GIC Redistributor register offsets (relative to GICR_BASE)
pub const GICR_STRIDE: u64 = 0x20000;        // Stride between redistributors
pub const GICR_CTLR: u64 = 0x0000;           // Redistributor Control Register
pub const GICR_IIDR: u64 = 0x0004;           // Implementer Identification Register
pub const GICR_TYPER: u64 = 0x0008;          // Redistributor Type Register
pub const GICR_STATUSR: u64 = 0x0010;        // Error Reporting Status Register
pub const GICR_WAKER: u64 = 0x0014;          // Redistributor Wakeup Control Register

// SGI_base frame (offset 0x10000 from Redistributor base)
pub const GICR_SGI_OFFSET: u64 = 0x10000;    // Offset to SGI_base frame
pub const GICR_IGROUPR0: u64 = 0x0080;       // Interrupt Group Register (SGIs/PPIs)
pub const GICR_ISENABLER0: u64 = 0x0100;     // Interrupt Set-Enable Register (SGIs/PPIs)
pub const GICR_ICENABLER0: u64 = 0x0180;     // Interrupt Clear-Enable Register (SGIs/PPIs)
pub const GICR_ISPENDR0: u64 = 0x0200;       // Interrupt Set-Pending Register (SGIs/PPIs)
pub const GICR_ICPENDR0: u64 = 0x0280;       // Interrupt Clear-Pending Register (SGIs/PPIs)
pub const GICR_ISACTIVER0: u64 = 0x0300;     // Interrupt Set-Active Register (SGIs/PPIs)
pub const GICR_ICACTIVER0: u64 = 0x0380;     // Interrupt Clear-Active Register (SGIs/PPIs)
pub const GICR_IPRIORITYR: u64 = 0x0400;     // Interrupt Priority Registers (SGIs/PPIs)
pub const GICR_ICFGR0: u64 = 0x0C00;         // Interrupt Configuration Register 0 (SGIs)
pub const GICR_ICFGR1: u64 = 0x0C04;         // Interrupt Configuration Register 1 (PPIs)
pub const GICR_IGRPMODR0: u64 = 0x0D00;      // Interrupt Group Modifier Register (SGIs/PPIs)

/// Constants for MPIDR register processing
pub const MPIDR_AFFINITY_MASK: u64 = 0xff00ff_ffff;
pub const MPIDR_MT_MASK: u64 = 1 << 24;
pub const MPIDR_AFFLVL_MASK: u64 = 0xff;
pub const MPIDR_AFFINITY_BITS: u32 = 8;
pub const MPIDR_CPU_MASK: u64 = MPIDR_AFFLVL_MASK;
pub const MPIDR_CLUSTER_MASK: u64 = MPIDR_AFFLVL_MASK << MPIDR_AFFINITY_BITS;
pub const MPIDR_AFF1_SHIFT: u32 = MPIDR_AFFINITY_BITS;
pub const S32_MPIDR_CPU_MASK_BITS: u32 = 3; // Log2 of max CPUs per cluster (8)

/// ICC_SRE_EL1 bits
pub const ICC_SRE_EL1_SRE: u64 = 1 << 0;     // System Register Enable
pub const ICC_SRE_EL1_DFB: u64 = 1 << 1;     // Disable FIQ Bypass
pub const ICC_SRE_EL1_DIB: u64 = 1 << 2;     // Disable IRQ Bypass
pub const ICC_SRE_EL1_EN: u64 = 1 << 3;      // Enable for Non-secure state

/// ICC_IGRPEN1_EL1 bits
pub const ICC_IGRPEN1_EL1_ENABLE: u64 = 1 << 0; // Enable for Group 1 interrupts

/// GICD_CTLR bits
pub const GICD_CTLR_ENABLE_G0: u32 = 1 << 0; // Enable Group 0 interrupts
pub const GICD_CTLR_ENABLE_G1NS: u32 = 1 << 1; // Enable Non-secure Group 1 interrupts
pub const GICD_CTLR_ENABLE_G1S: u32 = 1 << 2; // Enable Secure Group 1 interrupts
pub const GICD_CTLR_ARE_NS: u32 = 1 << 4; // Affinity Routing Enable for Non-secure state
pub const GICD_CTLR_ARE_S: u32 = 1 << 5; // Affinity Routing Enable for Secure state
pub const GICD_CTLR_DS: u32 = 1 << 6; // Disable Security

/// GICR_WAKER bits
pub const GICR_WAKER_PROCESSOR_SLEEP: u32 = 1 << 1; // Processor sleep bit
pub const GICR_WAKER_CHILDREN_ASLEEP: u32 = 1 << 2; // Children asleep bit

/// Platform-specific constants
pub const PLATFORM_CLUSTER_COUNT: u32 = 4;
pub const PLATFORM_MAX_CPUS_PER_CLUSTER: u32 = 8;

/// Track if GIC has been initialized
static GIC_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// GICv3 Driver for S32G3 secondary cores
pub struct GicV3Driver;

impl GicV3Driver {
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
    
    /// Get the current CPU's MPIDR_EL1 value
    #[inline(always)]
    pub fn get_mpidr() -> u64 {
        let mpidr: u64;
        unsafe {
            asm!(
                "mrs {0}, mpidr_el1",
                out(reg) mpidr
            );
        }
        mpidr
    }
    
    /// Calculate the Redistributor base address for the current core
    fn get_gicr_base_for_core() -> u64 {
        // In GICv3, each CPU has its own Redistributor
        // We need to find the right one based on the core's affinity
        
        // Get current core's affinity
        let mpidr = Self::get_mpidr();
        
        // Calculate core index
        let core_pos = match Self::plat_core_pos_by_mpidr(mpidr) {
            Ok(pos) => pos,
            Err(_) => 0, // Default to first core on error
        };
        
        // Get base address of this core's Redistributor
        GICR_BASE + (core_pos as u64 * GICR_STRIDE)
    }
    
    /// Enable the System Register interface for GICv3
    fn enable_system_registers() {
        unsafe {
            // Enable system register access
            let mut sre: u64;
            asm!(
                "mrs {0}, S3_0_C12_C12_5", // ICC_SRE_EL1
                out(reg) sre
            );
            
            // Set SRE bit to enable system register interface
            sre |= ICC_SRE_EL1_SRE;
            
            asm!(
                "msr S3_0_C12_C12_5, {0}", // ICC_SRE_EL1
                "isb",
                in(reg) sre
            );
            
            // Ensure changes are visible
            asm!("isb");
        }
    }
    
    /// Enable GICv3 Group 1 interrupts via ICC_IGRPEN1_EL1
    fn enable_group1_interrupts() {
        unsafe {
            // Enable Group 1 interrupts
            asm!(
                "msr S3_0_C12_C12_7, {0}", // ICC_IGRPEN1_EL1
                "isb",
                in(reg) ICC_IGRPEN1_EL1_ENABLE
            );
        }
    }
    
    /// Set the priority mask register (PMR)
    fn set_priority_mask(priority: u64) {
        unsafe {
            asm!(
                "msr S3_0_C4_C6_0, {0}", // ICC_PMR_EL1
                "isb",
                in(reg) priority
            );
        }
    }
    
    /// Set the binary point register (BPR)
    fn set_binary_point(bpr: u64) {
        unsafe {
            asm!(
                "msr S3_0_C12_C12_3, {0}", // ICC_BPR1_EL1
                "isb",
                in(reg) bpr
            );
        }
    }
    
    /// Wake up the Redistributor for this core
    fn wake_redistributor(gicr_base: u64) {
        unsafe {
            // Read current WAKER register state
            let waker = read_volatile((gicr_base + GICR_WAKER) as *const u32);
            
            // Clear ProcessorSleep bit to wake up the redistributor
            if (waker & GICR_WAKER_PROCESSOR_SLEEP) != 0 {
                let new_waker = waker & !GICR_WAKER_PROCESSOR_SLEEP;
                write_volatile((gicr_base + GICR_WAKER) as *mut u32, new_waker);
                
                // Wait until redistributor is awake
                loop {
                    let status = read_volatile((gicr_base + GICR_WAKER) as *const u32);
                    if (status & GICR_WAKER_CHILDREN_ASLEEP) == 0 {
                        break;
                    }
                    // Simple delay - in a real system you might want a timeout
                    for _ in 0..1000 {
                        core::hint::spin_loop();
                    }
                }
            }
        }
    }
    
    /// Initialize SGIs and PPIs for this core
    fn init_sgi_ppi(gicr_base: u64) {
        unsafe {
            let sgi_base = gicr_base + GICR_SGI_OFFSET;
            
            // Set all SGIs and PPIs to Group 1
            write_volatile((sgi_base + GICR_IGROUPR0) as *mut u32, 0xFFFFFFFF);
            
            // Set priority for SGIs and PPIs (lower value = higher priority)
            for i in 0..32 {
                let offset = GICR_IPRIORITYR + (i / 4) * 4;
                let shift = (i % 4) * 8;
                let addr = sgi_base + offset;
                
                let prio = read_volatile(addr as *const u32);
                let new_prio = (prio & !(0xFF << shift)) | (0x80 << shift); // Priority 0x80 (mid-range)
                write_volatile(addr as *mut u32, new_prio);
            }
            
            // Enable all SGIs and select PPIs
            write_volatile((sgi_base + GICR_ISENABLER0) as *mut u32, 0xFFFFFFFF);
        }
    }

    /// Initialize GICv3 for a secondary core
    pub fn init_secondary_core() {
        // Check if already initialized
        if GIC_INITIALIZED.load(Ordering::Relaxed) {
            return;
        }
        
        // First, enable the System Register interface
        Self::enable_system_registers();
        
        // Get the Redistributor base address for this core
        let gicr_base = Self::get_gicr_base_for_core();
        
        // Wake up the Redistributor
        Self::wake_redistributor(gicr_base);
        
        // Initialize SGIs and PPIs
        Self::init_sgi_ppi(gicr_base);
        
        // Set priority mask to allow all but the highest priority interrupts
        Self::set_priority_mask(0xF0);
        
        // Set binary point for group priority
        Self::set_binary_point(0);
        
        // Enable Group 1 interrupts
        Self::enable_group1_interrupts();
        
        // Mark as initialized
        GIC_INITIALIZED.store(true, Ordering::Relaxed);
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

    /// Acknowledge an interrupt (get its ID)
    #[inline(always)]
    pub fn acknowledge_interrupt() -> u32 {
        let iar: u64;
        unsafe {
            asm!(
                "mrs {0}, S3_0_C12_C12_0", // ICC_IAR1_EL1
                out(reg) iar
            );
        }
        iar as u32 & 0xFFFFFF // Mask to get interrupt ID bits
    }

    /// End an interrupt
    #[inline(always)]
    pub fn end_interrupt(interrupt_id: u32) {
        unsafe {
            asm!(
                "msr S3_0_C12_C12_1, {0}", // ICC_EOIR1_EL1
                "isb",
                in(reg) interrupt_id as u64
            );
        }
    }
    
    /// Set priority for an SGI interrupt
    pub fn set_sgi_priority(gicr_base: u64, sgi_id: u8, priority: u8) {
        if sgi_id > 15 {
            return; // Invalid SGI ID
        }
        
        unsafe {
            let sgi_base = gicr_base + GICR_SGI_OFFSET;
            let byte_offset = sgi_id as u64;
            let reg_addr = sgi_base + GICR_IPRIORITYR + byte_offset;
            
            // Each SGI has a byte-sized priority field
            write_volatile(reg_addr as *mut u8, priority);
        }
    }
    
    /// Generate a software interrupt to a core using affinity routing
    pub fn send_sgi_to_core(target_core_pos: u32, sgi_id: u8) -> Result<(), &'static str> {
        if sgi_id > 15 {
            return Err("Invalid SGI ID: must be 0-15");
        }
        
        let cpu_id = target_core_pos & ((1 << S32_MPIDR_CPU_MASK_BITS) - 1);
        let cluster_id = target_core_pos >> S32_MPIDR_CPU_MASK_BITS;
        
        // Validate CPU can be represented in 4-bit Aff0 field
        if cpu_id > 15 {
            return Err("CPU ID exceeds maximum Aff0 value (15)");
        }
        
        // Correct register format:
        // [63:48] - Aff3 (0)
        // [47:44] - Reserved (0)
        // [40]    - IRM (0 for targeted SGI)
        // [39:32] - Aff2 (0)
        // [31:24] - INTID (SGI ID)
        // [23:16] - Aff1 (Cluster)
        // [15:0]  - Target List (1 << CPU)
        let sgi_value = ((sgi_id as u64) << 24) |
                        ((cluster_id as u64) << 16) |
                        (1u64 << cpu_id);
        
        unsafe {
            asm!(
                "msr S3_0_C12_C11_5, {0}", // ICC_SGI1R_EL1
                "isb",
                in(reg) sgi_value
            );
        }
        
        Ok(())
    }
    
    /// Send SGI to all cores in the system
    pub fn send_sgi_to_all(sgi_id: u8) -> Result<(), &'static str> {
        if sgi_id > 15 {
            // SGIs are IDs 0-15
            return Err("Invalid SGI ID: must be 0-15");
        }
        
        // To target all CPUs in GICv3:
        // - Set the IRM bit (bit 31) = 1 (1 << 31)
        // - Target all security states (bit 30) = 0
        // - Include sgi_id in bits 0-3
        let sgi_value: u64 = (1u64 << 31) | (sgi_id as u64);
        
        unsafe {
            asm!(
                "msr S3_0_C12_C11_5, {0}", // ICC_SGI1R_EL1
                "isb",
                in(reg) sgi_value
            );
        }
        
        Ok(())
    }
    
    /// Send SGI to all cores except the current one
    pub fn send_sgi_to_others(sgi_id: u8) -> Result<(), &'static str> {
        if sgi_id > 15 {
            // SGIs are IDs 0-15
            return Err("Invalid SGI ID: must be 0-15");
        }
        
        // To target all CPUs except self in GICv3:
        // - Set the IRM bit (bit 31) = 1 (1 << 31)
        // - Set the exclude-self bit (bit 29) = 1 (1 << 29)
        // - Include sgi_id in bits 0-3
        let sgi_value: u64 = (1u64 << 31) | (1u64 << 29) | (sgi_id as u64);
        
        unsafe {
            asm!(
                "msr S3_0_C12_C11_5, {0}", // ICC_SGI1R_EL1
                "isb",
                in(reg) sgi_value
            );
        }
        
        Ok(())
    }
    
    /// Enable a specific SPI interrupt
    pub fn enable_spi(interrupt_id: u32) -> Result<(), &'static str> {
        if interrupt_id < 32 || interrupt_id >= 1020 {
            return Err("Invalid SPI ID: must be 32-1019");
        }
        
        unsafe {
            let reg_offset = ((interrupt_id / 32) * 4) as u64;
            let bit_offset = interrupt_id % 32;
            let reg_addr = GICD_BASE + GICD_ISENABLER + reg_offset;
            
            let value = 1u32 << bit_offset;
            write_volatile(reg_addr as *mut u32, value);
        }
        
        Ok(())
    }
    
    /// Disable a specific SPI interrupt
    pub fn disable_spi(interrupt_id: u32) -> Result<(), &'static str> {
        if interrupt_id < 32 || interrupt_id >= 1020 {
            return Err("Invalid SPI ID: must be 32-1019");
        }
        
        unsafe {
            let reg_offset = ((interrupt_id / 32) * 4) as u64;
            let bit_offset = interrupt_id % 32;
            let reg_addr = GICD_BASE + GICD_ICENABLER + reg_offset;
            
            let value = 1u32 << bit_offset;
            write_volatile(reg_addr as *mut u32, value);
        }
        
        Ok(())
    }
    
    /// Set priority for an SPI interrupt
    pub fn set_spi_priority(interrupt_id: u32, priority: u8) -> Result<(), &'static str> {
        if interrupt_id < 32 || interrupt_id >= 1020 {
            return Err("Invalid SPI ID: must be 32-1019");
        }
        
        unsafe {
            let byte_offset = interrupt_id as u64;
            let reg_addr = GICD_BASE + GICD_IPRIORITYR + byte_offset;
            
            write_volatile(reg_addr as *mut u8, priority);
        }
        
        Ok(())
    }
    
    /// Set the target for an SPI interrupt using affinity routing
    pub fn set_spi_target(interrupt_id: u32, target_aff: u64) -> Result<(), &'static str> {
        if interrupt_id < 32 || interrupt_id >= 1020 {
            return Err("Invalid SPI ID: must be 32-1019");
        }
        
        unsafe {
            let reg_addr = GICD_BASE + GICD_IROUTER + (interrupt_id as u64) * 8;
            write_volatile(reg_addr as *mut u64, target_aff);
        }
        
        Ok(())
    }
}