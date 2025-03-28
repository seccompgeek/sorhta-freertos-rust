pub mod aarch64;
pub mod s32g3;
pub mod gic;
pub mod exceptions;

// Interrupt related functions
pub fn enable_interrupt(irq_num: u32) {
    gic::enable_interrupt(irq_num);
}

pub fn disable_interrupt(irq_num: u32) {
    gic::disable_interrupt(irq_num);
}

pub fn get_interrupt_id() -> u32 {
    gic::get_interrupt_id()
}

pub fn end_of_interrupt(irq_num: u32) {
    gic::end_of_interrupt(irq_num);
}

pub fn set_interrupt_priority(irq_num: u32, priority: u8) {
    gic::set_priority(irq_num, priority);
}

pub fn send_sgi(sgi_id: u32, target_list: u8) {
    gic::send_sgi(sgi_id, target_list, 0);
} 

// CPU core functions
pub fn enable_interrupts() {
    unsafe {
        aarch64::enable_irq();
    }
}

pub fn disable_interrupts() {
    unsafe {
        aarch64::disable_irq();
    }
}

pub fn wait_for_interrupt() {
    aarch64::wfi();
}

pub fn cpu_id() -> u8 {
    aarch64::cpu_id()
}

pub fn current_el() -> u8 {
    aarch64::current_el()
}

// Memory barrier functions
pub fn dsb() {
    aarch64::dsb();
}

pub fn isb() {
    aarch64::isb();
}

// Time-related functions
pub fn get_system_tick() -> u64 {
    s32g3::timer::get_system_ticks()
}

pub fn delay_us(us: u32) {
    s32g3::timer::delay_us(us);
}

pub fn delay_ms(ms: u32) {
    s32g3::timer::delay_ms(ms);
}

// Hardware initialization
pub fn init() {
    s32g3::init();
    gic::init();  // Initialize GIC for this core
}