use core::mem::size_of;
use core::ptr::{self, read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, AtomicU8};
use core::{mem, ptr::slice_from_raw_parts_mut};

use aarch64_cpu::registers::{ESR_EL1::EC::WatchpointLowerEL, PAR_EL1::PA};
use alloc::vec::Vec;
use shmem::{Request, NUM_REQUEST_CORES, REQUEST_COMPLETED, REQUEST_FAILED, REQUEST_MEMORY_BASE, REQUEST_READ, REQUEST_TAKEN, REQUEST_VALID, REQUEST_WRITE, SHMEM_BASE};
use core::sync::atomic::Ordering;
mod shmem;

use crate::arch::dsb;
use crate::services::shmem::Settings;

const TEMP_REQUEST_MEM: [u8; NUM_REQUEST_CORES*size_of::<Request>()] = [0; NUM_REQUEST_CORES*size_of::<Request>()];

pub struct Worker {

}

impl Worker {
    pub fn init() {
        let shmem_base_ptr = SHMEM_BASE as *mut u8;
        let settings = unsafe { &mut *(shmem_base_ptr as *mut Settings) };

        let requests = Self::get_request_mem();
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
        let mut available_requests = 0u64; // Bitset for tracking available requests
        let requests = Worker::get_request_mem();
        
        // First pass: Identify and prepare valid requests
        for (_, req) in requests.iter_mut().enumerate() {
            // Atomically check and update status from VALID to TAKEN
            if let Ok(_) = req.status.compare_exchange(
                REQUEST_VALID, 
                REQUEST_TAKEN, 
                Ordering::Acquire, 
                Ordering::Relaxed
            ) {
                // Copy request data to temp storage
                match req.kind.load(Ordering::Acquire) {
                    REQUEST_READ => {
                        let mut read_count = 0;
                        let total = req.buf_size.load(Ordering::Acquire);
                        let addr = req.buf_addr.load(Ordering::Acquire); // Assuming 4 bytes per u32
                        while read_count < total {
                            // Properly handle addressing for 32-bit words
                            req.result[read_count].store(unsafe {read_volatile(addr as *const u32)}, Ordering::Release); 
                            read_count += 1;
                        }
                        // Use Release ordering to ensure all reads are completed before status update
                        req.status.store(REQUEST_COMPLETED, Ordering::Release);
                    },
                    
                    REQUEST_WRITE => {
                        let mut write_count = 0;
                        let total = req.buf_size.load(Ordering::Acquire);
                        let addr = req.buf_addr.load(Ordering::Acquire);
                        while write_count < total {
                            unsafe {
                                write_volatile((addr + write_count) as *mut u32, req.result[write_count].load(Ordering::Acquire));
                            }
                            write_count += 1;
                        }
                        // Use Release ordering to ensure all writes are visible before status update
                        req.status.store(REQUEST_COMPLETED, Ordering::Release);
                    },
                    
                    _ => {
                        // Invalid request type
                        req.kind.store(req.kind.load(Ordering::Acquire), Ordering::Release); 
                        req.result[0].store(req.kind.load(Ordering::Acquire), Ordering::Release);
                        req.status.store(REQUEST_FAILED, Ordering::Release);
                    }
                }
                // temp_requests[index].buf_addr.store(req.buf_addr.load(Ordering::Acquire), Ordering::Release);
                // temp_requests[index].buf_size.store(req.buf_size.load(Ordering::Acquire), Ordering::Release);
                // temp_requests[index].kind.store(req.kind.load(Ordering::Acquire), Ordering::Release);
                // //temp_requests[index].result.copy_from_slice(&req.result);
                
                // // Set the bit corresponding to this index
                // available_requests |= 1u64 << index;
                dsb();
            }
        }
        
        // // Second pass: Process the identified requests
        // // Only process indices that have their bit set in available_requests
        // for index in 0..NUM_REQUEST_CORES {
        //     if (available_requests & (1u64 << index)) != 0 {
        //         let req = &temp_requests[index];
        //         match req.kind.load(Ordering::Acquire) {
        //             REQUEST_READ => {
        //                 let mut read_count = 0;
        //                 let total = req.buf_size.load(Ordering::Acquire);
        //                 while read_count < total {
        //                     // Properly handle addressing for 32-bit words
        //                     let addr = req.buf_addr.load(Ordering::Acquire) as usize; // Assuming 4 bytes per u32
        //                     requests[index].result[read_count].store(unsafe {read_volatile(addr as *const u32)}, Ordering::Release); 
        //                     read_count += 1;
        //                 }
        //                 // Use Release ordering to ensure all reads are completed before status update
        //                 requests[index].status.store(REQUEST_COMPLETED, Ordering::Release);
        //             },
                    
        //             REQUEST_WRITE => {
        //                 let mut write_count = 0;
        //                 let total = req.buf_size.load(Ordering::Acquire);
        //                 while write_count < total {
        //                     // Properly handle addressing for 32-bit words
        //                     let addr = req.buf_addr.load(Ordering::Acquire) as usize; // Assuming 4 bytes per u32
        //                     unsafe {
        //                         write_volatile(addr as *mut u32, req.result[write_count].load(Ordering::Acquire));
        //                     }
        //                     write_count += 1;
        //                 }
        //                 // Use Release ordering to ensure all writes are visible before status update
        //                 requests[index].status.store(REQUEST_COMPLETED, Ordering::Release);
        //             },
                    
        //             _ => {
        //                 // Invalid request type
        //                 requests[index].kind.store(req.kind.load(Ordering::Acquire), Ordering::Release); 
        //                 requests[index].result[0].store(req.kind.load(Ordering::Acquire), Ordering::Release);
        //                 requests[index].status.store(REQUEST_FAILED, Ordering::Release);
        //             }
        //         }
                
        //         // A single DSB after each request is completed
        //         dsb();
        //     }
        // }
    }

}