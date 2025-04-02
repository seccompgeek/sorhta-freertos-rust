pub mod uart;
pub mod shmem;

// Initialize all drivers
pub fn init() {
    uart::init();
}

