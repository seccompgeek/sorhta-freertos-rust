// Cortex-A53 specific cache line size (64 bytes)
const CACHE_LINE_SIZE: usize = 64;

/// Flush (clean and invalidate) data cache range
/// 
/// This function will clean and invalidate the data cache for the memory range
/// from `start` to `end`. This is useful when memory is shared with devices
/// or other CPU cores that don't participate in cache coherency.
///
/// # Safety
///
/// This function is unsafe because it manipulates CPU cache state which
/// can cause data corruption if used incorrectly.
#[no_mangle]
pub unsafe extern "C" fn flush_dcache_range(mut start: usize, end: usize) {
    // Align start address down to cache line boundary
    start &= !(CACHE_LINE_SIZE - 1);
    
    // Process each cache line in the range
    while start < end {
        // Assembly instruction: dc civac, x0
        // Clean and Invalidate data cache line by VA to Point of Coherency
        core::arch::asm!(
            "dc civac, {0}",
            in(reg) start,
            options(nostack)
        );
        
        // Move to next cache line
        start += CACHE_LINE_SIZE;
    }
    
    // Ensure completion of the cache maintenance operations
    dsb();
}

/// Clean data cache range (without invalidation)
/// 
/// This function will clean (write back) the data cache for the memory range
/// from `start` to `end` without invalidating it. This is useful when data
/// needs to be visible to other observers but will still be used by this core.
///
/// # Safety
///
/// This function is unsafe because it manipulates CPU cache state which
/// can cause data corruption if used incorrectly.
#[no_mangle]
pub unsafe extern "C" fn clean_dcache_range(mut start: usize, end: usize) {
    // Align start address down to cache line boundary
    start &= !(CACHE_LINE_SIZE - 1);
    
    // Process each cache line in the range
    while start < end {
        // Assembly instruction: dc cvac, x0
        // Clean data cache line by VA to Point of Coherency
        core::arch::asm!(
            "dc cvac, {0}",
            in(reg) start,
            options(nostack)
        );
        
        // Move to next cache line
        start += CACHE_LINE_SIZE;
    }
    
    // Ensure completion of the cache maintenance operations
    dsb();
}

/// Invalidate data cache range (without cleaning)
/// 
/// This function will invalidate the data cache for the memory range
/// from `start` to `end` without cleaning it. This is useful when data
/// will be overwritten entirely or when you need to ensure fresh data
/// is loaded from memory.
///
/// # Safety
///
/// This function is unsafe because it manipulates CPU cache state which
/// can cause data corruption if used incorrectly. Only use on memory ranges
/// that don't contain dirty data in the cache.
#[no_mangle]
pub unsafe extern "C" fn invalidate_dcache_range(mut start: usize, end: usize) {
    // Align start address down to cache line boundary
    start &= !(CACHE_LINE_SIZE - 1);
    
    // Process each cache line in the range
    while start < end {
        // Assembly instruction: dc ivac, x0
        // Invalidate data cache line by VA to Point of Coherency
        core::arch::asm!(
            "dc ivac, {0}",
            in(reg) start,
            options(nostack)
        );
        
        // Move to next cache line
        start += CACHE_LINE_SIZE;
    }
    
    // Ensure completion of the cache maintenance operations
    dsb();
}

/// Data Synchronization Barrier
/// 
/// Ensures all cache maintenance operations are complete and visible.
#[inline(always)]
fn dsb() {
    unsafe {
        core::arch::asm!("dsb sy", options(nostack));
    }
}

// Memory regions - specific to S32G3 memory map
pub mod memory {
    // DDR memory region (our code lives here)
    pub const DDR_START: usize = 0xE0000000;
    pub const DDR_END: usize = 0xE2000000;  // 32MB region
    
    // SRAM memory region
    pub const SRAM_START: usize = 0x34000000;
    pub const SRAM_END: usize = 0x40000000;  // 64MB region
    
    // Device memory regions (peripherals, etc.)
    pub const DEVICE_START: usize = 0x40000000;
    pub const DEVICE_END: usize = 0x80000000;
    
    // Constants for MMU page table entries
    pub const PAGE_SIZE_4K: usize = 0x1000;
    pub const PAGE_SIZE_2M: usize = 0x200000;
    pub const PAGE_SIZE_1G: usize = 0x40000000;
    
    // Function to check if address is in device memory (non-cacheable)
    #[inline]
    pub fn is_device_memory(addr: usize) -> bool {
        addr >= DEVICE_START && addr < DEVICE_END
    }
    
    // Function to check if address is in our executable DDR region
    #[inline]
    pub fn is_ddr_memory(addr: usize) -> bool {
        addr >= DDR_START && addr < DDR_END
    }
    
    // Function to check if address is in SRAM
    #[inline]
    pub fn is_sram_memory(addr: usize) -> bool {
        addr >= SRAM_START && addr < SRAM_END
    }
    
    // Function to check if memory is cacheable
    #[inline]
    pub fn is_cacheable_memory(addr: usize) -> bool {
        is_ddr_memory(addr) || is_sram_memory(addr)
    }
}

// Re-export commonly used constants at the module level for convenience
pub use memory::{DDR_START, DDR_END, DEVICE_START, DEVICE_END, PAGE_SIZE_4K, PAGE_SIZE_2M};