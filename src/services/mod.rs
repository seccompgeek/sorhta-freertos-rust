use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::AtomicU8;
use core::{mem, ptr::slice_from_raw_parts_mut};

use aarch64_cpu::registers::{ESR_EL1::EC::WatchpointLowerEL, PAR_EL1::PA};
use alloc::vec::Vec;
use shmem::{Request, CORE_REQUEST_FLAGS_BASE, NUM_REQUEST_CORES, REQUEST_COMPLETED, REQUEST_FAILED, REQUEST_MEMORY_BASE, REQUEST_READ, REQUEST_TAKEN, REQUEST_VALID, REQUEST_WRITE};
use core::sync::atomic::Ordering;
mod shmem;

use crate::drivers::shmem::SHMEM_BASE;
use crate::services::shmem::Settings;

const TEMP_REQUEST_MEM: [u8; NUM_REQUEST_CORES*size_of::<Request>()] = [0; NUM_REQUEST_CORES*size_of::<Request>()];

pub struct Worker {

}

impl Worker {
    pub fn init() {
        let shmem_base_ptr = SHMEM_BASE as *mut u8;
        let settings = unsafe { &mut *(shmem_base_ptr as *mut Settings) };

        let core_request_flags = unsafe {&mut *slice_from_raw_parts_mut(CORE_REQUEST_FLAGS_BASE as *mut AtomicU8, NUM_REQUEST_CORES)};
        for core_flag in core_request_flags{
            *core_flag = AtomicU8::new(0);
        }

        let requests = unsafe {&mut *slice_from_raw_parts_mut(REQUEST_MEMORY_BASE as *mut Request, NUM_REQUEST_CORES)};
        for req in requests {
            req.initialize();
        }

        settings.initialize();
    }

    fn get_temp_request_mem() -> &'static mut [Request] {
        let temp_requests = unsafe {&mut *(slice_from_raw_parts_mut(TEMP_REQUEST_MEM.as_ptr() as *mut Request, NUM_REQUEST_CORES))};
        temp_requests
    }

    fn get_request_mem() -> &'static mut [Request] {
        let temp_requests = unsafe {&mut *(slice_from_raw_parts_mut(REQUEST_MEMORY_BASE as *mut Request, NUM_REQUEST_CORES))};
        temp_requests
    }

    pub fn do_work() {
        let temp_requests = Worker::get_temp_request_mem();
        let mut available_requests = 0x0;
        let requests = Worker::get_request_mem();
        for (index, req) in requests.iter_mut().enumerate() {
            if req.status.load(Ordering::Relaxed) == REQUEST_VALID {
                temp_requests[index].buf_addr = req.buf_addr;
                temp_requests[index].buf_size = req.buf_size;
                temp_requests[index].kind = req.kind;
                temp_requests[index].result.copy_from_slice(&req.result);
            }
            if let Ok(_) = req.status.compare_exchange(REQUEST_VALID, REQUEST_TAKEN, Ordering::Acquire, Ordering::Relaxed) {
                available_requests |= 0x1 << index;
            }
        }

        let mut counter = 0;
        let mut iter = 0x1;
        while counter < NUM_REQUEST_CORES {
            if (iter & available_requests) != 0 {
                let req = &mut temp_requests[counter];
                match req.kind {
                    // READ
                    REQUEST_READ => {
                        let mut read_count = 0;
                        while read_count < req.buf_size {
                            req.result[read_count] = unsafe {read_volatile((req.buf_addr + read_count) as *const u32)};
                            read_count += 1;
                        }
                        req.status.store(REQUEST_COMPLETED, Ordering::Relaxed);
                    }

                    // WRITE
                    REQUEST_WRITE => {
                        let mut write_count = 0;
                        while write_count < req.buf_size {
                            unsafe {
                                write_volatile((req.buf_addr + write_count) as *mut u32, req.result[write_count]);
                                write_count += 1;
                            }
                        }
                        req.status.store(REQUEST_COMPLETED, Ordering::Relaxed);
                    }
                    _ => {
                        req.status.store(REQUEST_FAILED, Ordering::Relaxed);
                    }
                }
            }

            iter <<= 0x1;
            counter += 1;
        }
    }

}