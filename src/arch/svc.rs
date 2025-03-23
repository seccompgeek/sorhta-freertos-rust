use alloc::format;

// SVC (Supervisor Call) handler implementation for S32G3
use crate::drivers::uart;
use core::arch::asm;
use core::slice;
use core::str;

// SVC function numbers
pub const SVC_PRINT: u64 = 0x01;
pub const SVC_MEM_ALLOC: u64 = 0x02;
pub const SVC_MEM_FREE: u64 = 0x03;
pub const SVC_THREAD_CREATE: u64 = 0x04;
pub const SVC_MUTEX_LOCK: u64 = 0x05;
pub const SVC_MUTEX_UNLOCK: u64 = 0x06;

// Simple memory allocator state
static mut HEAP_START: u64 = 0;
static mut HEAP_SIZE: u64 = 0;
static mut HEAP_NEXT: u64 = 0;

// Basic memory allocation system
pub fn init_memory_allocator() {
    unsafe {
        extern "C" {
            static __heap_start: u64;
            static __heap_end: u64;
        }
        
        HEAP_START = &__heap_start as *const _ as u64;
        HEAP_SIZE = (&__heap_end as *const _ as u64) - HEAP_START;
        HEAP_NEXT = HEAP_START;
        
        uart::puts(&format!("Memory allocator initialized: start=0x{:x}, size=0x{:x}\n", 
                          HEAP_START, HEAP_SIZE));
    }
}

// Simple (and not thread-safe) memory allocation
fn mem_alloc(size: u64) -> u64 {
    unsafe {
        // Align to 8 bytes
        let aligned_size = (size + 7) & !7;
        
        if HEAP_NEXT + aligned_size > HEAP_START + HEAP_SIZE {
            // Out of memory
            return 0;
        }
        
        let allocation = HEAP_NEXT;
        HEAP_NEXT += aligned_size;
        
        allocation
    }
}

// Free memory (very simple, doesn't actually free anything in this implementation)
fn mem_free(addr: u64) -> u64 {
    // This is a no-op in our simple memory system
    // A real implementation would actually free the memory
    0
}

// Create a wrapper function to make SVC calls from Rust
#[inline(never)]
pub fn call(function_id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let result: u64;
    
    unsafe {
        asm!(
            "mov x8, {fn_id}",
            "svc #0",
            fn_id = in(reg) function_id,
            inout("x0") arg0 => result,
            in("x1") arg1,
            in("x2") arg2,
            out("x8") _,
        );
    }
    
    result
}

// The actual SVC handler called from the exception vector
#[no_mangle]
pub extern "C" fn handle_el0_sync(esr: u64, _far: u64) -> u64 {
    // Extract the EC (Exception Class) from ESR
    let ec = (esr >> 26) & 0x3F;
    
    // Check if this is an SVC call (EC = 0x15 for SVC from AArch64)
    if ec == 0x15 {
        // Extract the SVC number from ESR
        let svc_num = esr & 0xFFFFFF;
        
        // Handle SVC calls using the registers saved by the exception handler
        unsafe {
            // We need to access the stack where registers were saved
            // This depends on the exact layout in our exception vector
            
            // Simplified approach for demo purposes - in a real system, 
            // we would carefully extract the saved x0-x3 values
            
            // For this example, we'll use inline assembly to get the values
            let function_id: u64;
            let arg0: u64;
            let arg1: u64;
            let arg2: u64;
            
            asm!(
                "ldr {0}, [sp, #0]",  // x0 is at the beginning of saved regs
                "ldr {1}, [sp, #16]", // x1 is 16 bytes from start
                "ldr {2}, [sp, #32]", // x2 is 32 bytes from start
                "ldr {3}, [sp, #64]", // x8 (function ID) is 64 bytes from start
                out(reg) arg0,
                out(reg) arg1,
                out(reg) arg2,
                out(reg) function_id,
            );
            
            // Now handle the SVC call
            handle_svc(function_id, arg0, arg1, arg2)
        }
    } else {
        // Not an SVC, handle other synchronous exceptions
        uart::puts(&format!("Unhandled synchronous exception: ESR=0x{:x}\n", esr));
        0
    }
}

// Handle SVC calls
fn handle_svc(function_id: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    uart::puts(&format!("SVC called: function_id=0x{:x}\n", function_id));
    
    match function_id {
        SVC_PRINT => {
            // Print a null-terminated string
            let ptr = arg0 as *const u8;
            let mut len = 0;
            
            // Find string length (careful with unsafe!)
            unsafe {
                while *ptr.add(len) != 0 {
                    len += 1;
                }
                
                // Convert to &str and print
                if let Ok(s) = str::from_utf8(slice::from_raw_parts(ptr, len)) {
                    uart::puts(s);
                    uart::puts("\n");
                } else {
                    uart::puts("SVC_PRINT: Invalid UTF-8 string\n");
                }
            }
            0
        },
        
        SVC_MEM_ALLOC => {
            // Allocate memory
            let size = arg0;
            uart::puts(&format!("SVC_MEM_ALLOC: size={}\n", size));
            mem_alloc(size)
        },
        
        SVC_MEM_FREE => {
            // Free memory
            let addr = arg0;
            uart::puts(&format!("SVC_MEM_FREE: addr=0x{:x}\n", addr));
            mem_free(addr)
        },
        
        SVC_THREAD_CREATE => {
            // Create a thread (simplified)
            let entry_point = arg0;
            let argument = arg1;
            uart::puts(&format!("SVC_THREAD_CREATE: entry=0x{:x}, arg=0x{:x}\n", 
                             entry_point, argument));
            // Thread creation implementation would go here
            0
        },
        
        SVC_MUTEX_LOCK => {
            // Lock a mutex
            let mutex_addr = arg0;
            uart::puts(&format!("SVC_MUTEX_LOCK: mutex=0x{:x}\n", mutex_addr));
            // Mutex lock implementation would go here
            0
        },
        
        SVC_MUTEX_UNLOCK => {
            // Unlock a mutex
            let mutex_addr = arg0;
            uart::puts(&format!("SVC_MUTEX_UNLOCK: mutex=0x{:x}\n", mutex_addr));
            // Mutex unlock implementation would go here
            0
        },
        
        _ => {
            uart::puts(&format!("Unknown SVC function: 0x{:x}\n", function_id));
            u64::MAX  // Return error code
        }
    }
}