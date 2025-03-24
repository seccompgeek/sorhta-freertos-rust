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

use arch::gic;
use drivers::uart::console_init;
use drivers::uart::print_init_complete;
use drivers::uart::print_init_message;
use drivers::uart::putc;
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

//global_asm!(include_str!("linflex_console.S"));
// Boot section assembly code
// ATF will load our image and jump to _start
global_asm!(include_str!("start.S"));
global_asm!(include_str!("exceptions.S"));

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
    // unsafe {
    //     extern "C" {
    //         static _heap_start: u64;
    //         static _heap_end: u64;
    //     }
        
    //     let heap_start = &_heap_start as *const u64 as *mut u8;
    //     let heap_end = &_heap_end as *const u64 as usize;
    //     let heap_size = heap_end - (heap_start as usize);
        
    //     ALLOCATOR.lock().init(heap_start, heap_size);
    // }

    // console_init();
    unsafe {
        let ptr = 0xE0100000 as *mut u32;
        *ptr = 0x1; // just to check that we have initialized properly
    }

    loop {
        
    }

    // console_init();
    // gic::init();
    // // Initialize S32G3 peripherals
    // arch::s32g3::init();
    // // Enable interrupts
    // arch::enable_interrupts();
    // //print_init_complete();

    
    // // Initialize the UART for our console output
    // //drivers::uart::init();
    
    // // Print initial hello message
    // //println!("\r\n\r\nS32G3 Cortex-A Rust port initializing...");

    // loop {
        
    // }
    // // Print CPU information
    // unsafe {
    //     let mut cpu_id: u64;
    //     asm!("mrs {}, mpidr_el1", out(reg) cpu_id);
    //     cpu_id &= 0xFF;
        
    //     let mut el: u64;
    //     asm!("mrs {}, CurrentEL", out(reg) el);
    //     el = (el >> 2) & 0x3;
        
    //     println!("Running on CPU {} at EL{}", cpu_id, el);
    // }
    
    // // Main loop that prints hello
    // let mut counter = 0;
    // loop {
    //     // Print hello
    //     println!("Hello, World from S32G3 Cortex-A in Rust! (count: {})", counter);
    //     counter += 1;
        
    //     // Use S32G3 timer for precise delay
    //     arch::s32g3::timer::delay_ms(1000);
    // }
}