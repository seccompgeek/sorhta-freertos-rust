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
use core::ptr::write_volatile;

use arch::aarch64::dsb;
use arch::enable_interrupts;
use arch::gic;
// use arch::gic::broadcast_custom_ipi;
// use arch::gic::request_ipi;
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

#[inline(always)]
pub unsafe fn smc_call(
    function_id: u32,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    let mut result: u64;
    
    #[cfg(target_arch = "aarch64")]
    {
        core::arch::asm!(
            "smc #0",
            inout("x0") function_id as u64 => result,
            in("x1") arg1,
            in("x2") arg2,
            in("x3") arg3,
            in("x4") arg4,
            in("x5") arg5,
            in("x6") arg6,
            options(nostack)
        );
    }
    
    result
}

unsafe fn ensure_memory_visible() {
    // Data Synchronization Barrier
    core::arch::asm!("dsb sy", options(nomem, nostack));
    
    // Data Memory Barrier
    core::arch::asm!("dmb sy", options(nomem, nostack));
}


#[no_mangle]
extern "C" fn kernel_init() -> ! {
    //Initialize the heap allocator
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
    arch::disable_interrupts();
    arch::init();
    arch::enable_interrupts();
    //console_init();
    //enable_interrupts();
    //gic::init();
    //enable_interrupts();
    //request_ipi(1);
    //  unsafe {
    // //     //ensure_memory_visible();
    //      smc_call(0x84000008, 0, 0, 0, 0, 0, 0);
    // //     let ptr = 0xE0100000 as *mut u32;
    // //     write_volatile(ptr, 0x1);
    // //     core::sync::atomic::fence(core::sync::atomic::Ordering::Release);
    // //     //*ptr = 0x1; // just to check that we have initialized properly
    // }

    //panic!();

    //let mut waiter = 0x110000;
    loop {
        // while waiter > 0 {
        //     waiter -= 1;
        // }
        // //broadcast_custom_ipi();
        let _ = gic::GicDriver::send_sgi(0xFF, 0x2);
        let _ = gic::GicDriver::send_sgi_to_core(3, 2);
        let _ = gic::GicDriver::send_sgi_to_core(4, 2);
        let _ = gic::GicDriver::send_sgi_to_core(5, 2);
        let _ = gic::GicDriver::send_sgi_to_core(6, 2);
        let ptr = 0xE0100000 as *mut u32;
        unsafe {
            *ptr = 0x1;
            dsb();
        }
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