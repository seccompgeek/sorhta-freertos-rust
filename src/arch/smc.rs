use alloc::format;
// SMC (Secure Monitor Call) handler implementation for S32G3
use crate::drivers::uart;
use crate::arch::CORE_STATES;
use core::arch::asm;
use core::sync::atomic::Ordering;

// SMC function numbers (ARM PSCI)
pub const SMC_PSCI_VERSION: u64 = 0x84000000;
pub const SMC_CPU_ON: u64 = 0x84000003;
pub const SMC_CPU_OFF: u64 = 0x84000002;
pub const SMC_SYSTEM_RESET: u64 = 0x84000009;
pub const SMC_AFFINITY_INFO: u64 = 0x84000004;

// Maximum number of cores on S32G3
const NUM_CORES: usize = 8;

// Data structure to store secondary core boot parameters
struct CoreBootParams {
    entry_point: u64,
    context_id: u64,
}

// Boot parameters for each core
static mut CORE_BOOT_PARAMS: [CoreBootParams; NUM_CORES] = [
    CoreBootParams { entry_point: 0, context_id: 0 },
    CoreBootParams { entry_point: 0, context_id: 0 },
    CoreBootParams { entry_point: 0, context_id: 0 },
    CoreBootParams { entry_point: 0, context_id: 0 },
    CoreBootParams { entry_point: 0, context_id: 0 },
    CoreBootParams { entry_point: 0, context_id: 0 },
    CoreBootParams { entry_point: 0, context_id: 0 },
    CoreBootParams { entry_point: 0, context_id: 0 },
];

// Create a wrapper function to make SMC calls from Rust
#[inline(never)]
pub fn call(function_id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let result: u64;
    
    unsafe {
        asm!(
            "smc #0",
            inout("x0") function_id => result,
            in("x1") arg0,
            in("x2") arg1,
            in("x3") arg2,
        );
    }
    
    result
}

// Function to get the current core ID
pub fn get_current_core_id() -> u64 {
    let mpidr: u64;
    unsafe {
        asm!("mrs {}, mpidr_el1", out(reg) mpidr);
    }
    mpidr & 0xFF
}

// The SMC handler called from the exception vector
#[no_mangle]
pub extern "C" fn handle_smc(function_id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    uart::puts(&format!("SMC called: function_id=0x{:x}\n", function_id));
    
    match function_id {
        SMC_PSCI_VERSION => {
            // Return PSCI version 1.1
            0x00010001
        },
        
        SMC_CPU_ON => {
            let target_cpu = arg0 & 0xFF;
            let entry_point = arg1;
            let context_id = arg2;
            
            uart::puts(&format!("CPU_ON: cpu={}, entry=0x{:x}, context=0x{:x}\n", 
                             target_cpu, entry_point, context_id));
            
            if target_cpu >= NUM_CORES as u64 {
                uart::puts("CPU_ON: Invalid CPU ID\n");
                return 0xFFFFFFFF; // Invalid parameters
            }
            
            if CORE_STATES[target_cpu as usize].load(Ordering::SeqCst) {
                uart::puts("CPU_ON: Core already active\n");
                return 0xFFFFFFFE; // Already on
            }
            
            // Store the entry point and context ID
            unsafe {
                CORE_BOOT_PARAMS[target_cpu as usize] = CoreBootParams {
                    entry_point,
                    context_id,
                };
            }
            
            // Platform-specific code to wake up secondary core would go here
            // For S32G3, this might involve writing to specific registers
            
            // For demonstration purposes, we'll simulate success
            // In reality, this needs to interact with hardware
            uart::puts("CPU_ON: Core activation initiated\n");
            0 // Success
        },
        
        SMC_CPU_OFF => {
            let this_cpu = get_current_core_id();
            
            uart::puts(&format!("CPU_OFF: cpu={}\n", this_cpu));
            
            // Mark core as inactive
            CORE_STATES[this_cpu as usize].store(false, Ordering::SeqCst);
            
            // Platform-specific code to power down current CPU would go here
            // This typically won't return
            
            uart::puts("CPU_OFF: Core deactivation simulated\n");
            0 // Success
        },
        
        SMC_SYSTEM_RESET => {
            uart::puts("SYSTEM_RESET: Resetting system...\n");
            
            // Platform-specific code to reset the system would go here
            // For S32G3, this would involve writing to the reset controller
            
            // This function typically never returns
            loop {
                // Wait for reset to take effect
            }
        },
        
        SMC_AFFINITY_INFO => {
            let target_cpu = arg0 & 0xFF;
            
            uart::puts(&format!("AFFINITY_INFO: cpu={}\n", target_cpu));
            
            if target_cpu >= NUM_CORES as u64 {
                return 0xFFFFFFFF; // Invalid parameters
            }
            
            if CORE_STATES[target_cpu as usize].load(Ordering::SeqCst) {
                0 // Core is ON
            } else {
                1 // Core is OFF
            }
        },
        
        _ => {
            uart::puts(&format!("Unknown SMC function: 0x{:x}\n", function_id));
            0xFFFFFFFF // Invalid function ID
        }
    }
}

// Function to get secondary core boot parameters
// Called during secondary core boot process
#[no_mangle]
pub fn get_secondary_boot_params(core_id: u64) -> (u64, u64) {
    if core_id < NUM_CORES as u64 {
        unsafe {
            let params = &CORE_BOOT_PARAMS[core_id as usize];
            (params.entry_point, params.context_id)
        }
    } else {
        (0, 0)
    }
}