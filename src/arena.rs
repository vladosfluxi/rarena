#![allow(dead_code)]
#![allow(unused_imports)]
extern crate alloc as core_alloc;

use core::ptr::{
    NonNull,
    write,
    write_bytes,
};

use core::mem::{
    align_of,
    size_of,
};

use core_alloc::alloc::{
    Layout,
    LayoutError,
    alloc,
    dealloc,
};

pub struct Arena {
    offset: usize,
    aligment: usize,
    cap: usize,
    ptr_buffer: *mut u8,
}

impl Arena {
    pub fn new() -> Self {
        let cap: usize = 4096;
        let layout = Layout::array::<u8>(cap).unwrap();

        let raw_ptr = unsafe { alloc(layout) };

        Self {
            offset: 0,
            aligment: 8,
            cap,
            ptr_buffer: raw_ptr,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::array::<u8>(cap).unwrap();

        let raw_ptr = unsafe { alloc(layout) };

        Self {
            offset: 0,
            aligment: 8,
            cap,
            ptr_buffer: raw_ptr,
        }
    }
}
