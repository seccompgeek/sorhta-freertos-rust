pub mod port;
pub mod tasks;
pub mod queue;

use crate::arch;

// Initialize the FreeRTOS system
pub fn init() {
    // Initialize FreeRTOS subsystems
    port::init();
    tasks::init();
    queue::init();
}

// Critical section management
pub fn enter_critical_section() {
    arch::disable_interrupts();
}

pub fn exit_critical_section() {
    arch::enable_interrupts();
}

// FreeRTOS system tick handler
// Would be called by timer interrupt
pub fn tick_handler() {
    let inside_isr = port::is_inside_isr();
    
    if inside_isr {
        tasks::increment_tick_from_isr();
    } else {
        tasks::increment_tick();
    }
}