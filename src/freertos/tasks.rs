use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use core::mem::MaybeUninit;
use crate::freertos::{enter_critical_section, exit_critical_section};
use crate::arch;
use alloc::vec::Vec;

// Simplified task control block
pub struct TCB {
    stack_pointer: *mut usize,
    priority: u8,
    name: &'static str,
    state: TaskState,
    stack_size: usize,
    function: fn(),
}

// Task states
#[derive(Copy, Clone, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Suspended,
}

// Task handle type
pub type TaskHandle = usize;

// System tick counter
static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

// Current running task
static CURRENT_TASK: AtomicUsize = AtomicUsize::new(0);

// Task list (simplified)
static mut TASKS: MaybeUninit<Vec<TCB>> = MaybeUninit::uninit();
static mut NUM_TASKS: usize = 0;

// Initialize the task subsystem
pub fn init() {
    unsafe {
        TASKS = MaybeUninit::new(Vec::new());
    }
}

// Create a new task
pub fn create_task(function: fn(), name: &'static str, stack_size: usize) -> TaskHandle {
    let task_id;
    
    enter_critical_section();
    
    unsafe {
        // Allocate stack (simplified)
        let stack = alloc::alloc::alloc(
            alloc::alloc::Layout::from_size_align(stack_size, 8).unwrap()
        ) as *mut usize;
        
        // Create TCB
        let tcb = TCB {
            stack_pointer: stack,
            priority: 1,
            name,
            state: TaskState::Ready,
            stack_size,
            function,
        };
        
        // Add to task list
        TASKS.assume_init_mut().push(tcb);
        task_id = NUM_TASKS;
        NUM_TASKS += 1;
    }
    
    exit_critical_section();
    
    task_id
}

// Start the scheduler
pub fn start_scheduler() {
    // This is a simplified implementation
    // In a real port, would set up timer interrupt and context switching
    
    if unsafe { NUM_TASKS == 0 } {
        // No tasks created
        return;
    }
    
    // Set first task as current
    CURRENT_TASK.store(0, Ordering::Relaxed);
    
    // Start first task
    unsafe {
        let task = &TASKS.assume_init_ref()[0];
        (task.function)();
    }
    
    // This should never be reached in a real implementation
}

// Get current task handle
pub fn get_current_task() -> TaskHandle {
    CURRENT_TASK.load(Ordering::Relaxed)
}

// Increment system tick
pub fn increment_tick() {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    check_delayed_tasks();
}

// Increment system tick from ISR
pub fn increment_tick_from_isr() {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    // In a real implementation, would defer task unblocking to the exit from ISR
}

// Get current tick count
pub fn get_tick_count() -> u64 {
    TICK_COUNT.load(Ordering::Relaxed)
}

// Delay the current task
pub fn delay(ticks: u32) {
    // For our simple implementation, we'll just busy-wait
    let start = get_tick_count();
    let target = start + ticks as u64;
    
    while get_tick_count() < target {
        // Yield to other tasks (in a real implementation)
        arch::wait_for_interrupt();
    }
}

// Check for tasks that should be unblocked
fn check_delayed_tasks() {
    // In a real implementation, would check for tasks whose delay has expired
    // and move them from the Blocked state to the Ready state
}