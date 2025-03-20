#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;

#[macro_use]
extern crate core;

use core::arch::global_asm;
use core::arch::asm;
use core::panic::PanicInfo;

// Import for heap allocator
use linked_list_allocator::LockedHeap;

// Define a global allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Single allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

mod arch;
mod drivers;
mod freertos;

// Boot section assembly code
// ATF will load our image and jump to _start
global_asm!(
    ".section .text.boot",
    ".global _start",
    "_start:",
    "   // Disable all interrupts",
    "   msr daifset, #0xf",
    "",
    "   // Set up stack pointer for each CPU core",
    "   mrs x1, mpidr_el1",
    "   and x1, x1, #0xFF        // Extract CPU ID",
    "   cbz x1, primary_core     // If CPU0, branch to primary core init",
    "",
    "secondary_cores:",
    "   // Secondary cores wait in a loop until woken",
    "1: wfe",
    "   b 1b",
    "",
    "primary_core:",
    "   // Set up stack pointer using ADRP",
    "   adrp x2, __stack_end",
    "   add x2, x2, :lo12:__stack_end",
    "   mov sp, x2",
    "",
    "   // Initialize floating point",
    "   mrs x0, cpacr_el1",
    "   orr x0, x0, #(3 << 20)", // Enable FP/SIMD
    "   msr cpacr_el1, x0",
    "",
    "   // Clear BSS section using ADRP",
    "   adrp x1, __bss_start",
    "   add x1, x1, :lo12:__bss_start",
    "   adrp x2, __bss_end",
    "   add x2, x2, :lo12:__bss_end",
    "   cmp x1, x2",
    "   b.eq 2f",
    "1: str xzr, [x1], #8",
    "   cmp x1, x2",
    "   b.lo 1b",
    "2:",
    "",
    "   // Invalidate caches",
    "   bl _invalidate_caches",
    "",
    "   // Jump to Rust code",
    "   bl kernel_init",
    "",
    "   // Should never reach here",
    "halt:",
    "   wfe",
    "   b halt",
    "",
    "// Cache invalidation routine",
    "_invalidate_caches:",
    "   // Invalidate instruction cache",
    "   ic ialluis",
    "   dsb ish",
    "   isb",
    "",
    "   // Return to caller",
    "   ret",
);

// Single panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\r\n\r\n*** PANIC ***");
    
    if let Some(location) = info.location() {
        println!("Location: {}:{}", location.file(), location.line());
    }
    
    if let Some(message) = info.message() {
        println!("Message: {}", message);
    }
    
    println!("\r\nSystem halted!");
    
    // Disable interrupts and enter infinite loop
    unsafe { arch::aarch64::disable_irq(); }
    
    loop {
        arch::aarch64::wfe();
    }
}

#[no_mangle]
extern "C" fn kernel_init() -> ! {
    // Initialize the heap allocator
    unsafe {
        extern "C" {
            static _heap_start: u64;
            static _heap_end: u64;
        }
        
        let heap_start = &_heap_start as *const u64 as *mut u8;
        let heap_end = &_heap_end as *const u64 as usize;
        let heap_size = heap_end - (heap_start as usize);
        
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
    
    // Initialize S32G3 peripherals
    arch::s32g3::init();
    
    // Print initial hello message
    println!("\r\n\r\nS32G3 Cortex-A Rust port initializing...");
    

    
    // Setup exception vectors
    arch::exceptions::init_vectors();
    
    // Print CPU information
    unsafe {
        let mut cpu_id: u64;
        asm!("mrs {}, mpidr_el1", out(reg) cpu_id);
        cpu_id &= 0xFF;
        
        let mut el: u64;
        asm!("mrs {}, CurrentEL", out(reg) el);
        el = (el >> 2) & 0x3;
        
        println!("Running on CPU {} at EL{}", cpu_id, el);
    }
    
    // Main loop that prints hello
    let mut counter = 0;
    loop {
        // Print hello
        println!("Hello, World from S32G3 Cortex-A in Rust! (count: {})", counter);
        counter += 1;
        
        // Use S32G3 timer for precise delay
        arch::s32g3::timer::delay_ms(1000);
    }
}