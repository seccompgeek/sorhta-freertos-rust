use core::{mem::{self, size_of}, sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering}};


pub const SHMEM_BASE: usize = 0xE200_0000;
pub const RESULT_BUFF_LENGTH: usize = 0x80;
pub const NUM_REQUEST_CORES: usize = 0x8;
pub const REQUEST_MEMORY_BASE: usize = SHMEM_BASE + (((mem::size_of::<Settings>() + size_of::<usize>()-1)/size_of::<usize>())) * size_of::<usize>();

pub const REQUEST_NONE: u32 = 0;
pub const REQUEST_VALID: u32 = 1;
pub const REQUEST_TAKEN: u32 = 2;
pub const REQUEST_COMPLETED: u32 = 3;
pub const REQUEST_FAILED: u32 = u32::MAX;

pub const REQUEST_WRITE: u32 = 1;
pub const REQUEST_READ: u32 = 2;

#[repr(C)]
pub struct Request {
    pub(crate) kind: u32,
    __padding: [u8;4],
    pub(crate) buf_addr: usize,
    pub(crate) buf_size: usize,
    pub(crate) result: [u32; RESULT_BUFF_LENGTH],
    pub(crate) status: AtomicU32,
}

#[repr(C)]
pub struct Settings {
    request_memory_base_addr: usize,
    initialized: AtomicU32,
}

impl Settings {
    pub fn initialize(&mut self) {
        self.initialized = AtomicU32::new(1);
        self.request_memory_base_addr = REQUEST_MEMORY_BASE;
    }
}


impl Request {
    pub fn initialize(&mut self) {
        self.status = AtomicU32::new(REQUEST_TAKEN);
    }
}