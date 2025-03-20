pub mod uart;

// Initialize all drivers
pub fn init() {
    uart::init();
}