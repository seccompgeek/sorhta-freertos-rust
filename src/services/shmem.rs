use core::{mem::{self, replace, size_of, size_of_val}, ptr::{self, slice_from_raw_parts_mut}, sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicU8, Ordering}};


pub const SHMEM_BASE: usize = 0xE200_0000;
pub const RESULT_BUFF_LENGTH: usize = 0x80;
pub const NUM_REQUEST_CORES: usize = 0x7;
pub const CORE_REQUEST_FLAGS_BASE: usize = SHMEM_BASE + ((mem::size_of::<Settings>() + size_of::<usize>()-1)/size_of::<usize>());

pub const REQUEST_MEMORY_BASE: usize = CORE_REQUEST_FLAGS_BASE + (((mem::size_of::<AtomicU8>()*NUM_REQUEST_CORES) + size_of::<usize>()-1)/size_of::<usize>());
pub const QUEUE_RESULTS_BASE: usize = 0xE208_000;

pub const REQUEST_NONE: u8 = 0;
pub const REQUEST_VALID: u8 = 1;
pub const REQUEST_TAKEN: u8 = 2;
pub const REQUEST_COMPLETED: u8 = 3;
pub const REQUEST_FAILED: u8 = 0xFF;

pub const REQUEST_WRITE: u8 = 1;
pub const REQUEST_READ: u8 = 2;

#[repr(C)]
pub struct Request {
    pub(crate) kind: u8,
    pub(crate) buf_addr: usize,
    pub(crate) buf_size: usize,
    pub(crate) result: [u32; RESULT_BUFF_LENGTH],
    pub(crate) status: AtomicU8,
}

#[repr(C)]
pub struct Settings {
    initialized: AtomicU8,
    request_memory_base_addr: usize,
    core_request_flags_base: usize
}

pub fn get_cores_requests_flag_buffer() -> &'static [AtomicU8]
{
    let ptr = slice_from_raw_parts_mut(CORE_REQUEST_FLAGS_BASE as *mut AtomicU8, NUM_REQUEST_CORES);
    unsafe {
        &mut *ptr
    }
}

impl Settings {
    pub fn initialize(&mut self) {
        self.initialized = AtomicU8::new(1);
        self.request_memory_base_addr = REQUEST_MEMORY_BASE;
        self.core_request_flags_base = CORE_REQUEST_FLAGS_BASE;
    }
}


impl Request {
    pub fn initialize(&mut self) {
        self.status = AtomicU8::new(REQUEST_NONE);
    }
}