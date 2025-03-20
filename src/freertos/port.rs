use core::sync::atomic::{AtomicBool, Ordering};
use crate::arch;

// Track if we're inside an ISR context
static IN_ISR: AtomicBool = AtomicBool::new(false);

// Initialize the port-specific features
pub fn init() {
    // Set up timer for system ticks (simplified for this example)
    // In a real implementation, would configure a hardware timer
}

// Check if currently in ISR/exception context
pub fn is_inside_isr() -> bool {
    IN_ISR.load(Ordering::Relaxed)
}

// Mark the start of ISR processing
pub fn enter_isr() {
    IN_ISR.store(true, Ordering::Relaxed);
}

// Mark the end of ISR processing
pub fn exit_isr() {
    IN_ISR.store(false, Ordering::Relaxed);
}

// Yield processor - trigger a context switch
pub fn yield_task() {
    // In a real implementation, would trigger SVC exception
    // For our minimal port, we'll simulate time slicing
    arch::wait_for_interrupt();
}

// Start the first task
pub fn start_first_task(sp: *const usize) {
    unsafe {
        // In a real implementation, would set up the stack and jump to the task
        // For our minimal port, we'll just call the task function directly
        let task_fn: fn() = core::mem::transmute(sp);
        task_fn();
    }
}