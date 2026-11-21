#![allow(dead_code)]
#![allow(unused_imports)]
#![no_std]
pub mod arena;
pub mod boxed;

extern crate alloc as core_alloc;

// use crate::arena::Arena;
use crate::arena::{
    Arena,
    Init,
    Uninit,
};

use core_alloc::alloc::{
    alloc,
    dealloc,
    handle_alloc_error,
    realloc,
};

use core::{
    alloc::Layout,
    ptr::NonNull,
};

pub struct Arenas {
    pub current: NonNull<Arena<Init>>,
    pub next_chunk_cap: usize,
}

impl Arenas {
    pub fn new() -> Self {
        let arena = Arena::new(None);
        let layout = Layout::new::<Arena<Init>>();
        let next_chunk_cap = arena.get_capacity() * 2;

        let raw = unsafe { alloc(layout) };
        let current = match NonNull::new(raw.cast::<Arena<Init>>()) {
            Some(ptr) => ptr,
            None => handle_alloc_error(layout),
        };

        unsafe {
            current.as_ptr().write(arena);
        }

        Self {
            current,
            next_chunk_cap,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let arena = Arena::with_capacity(cap, None);
        let layout = Layout::new::<Arena<Init>>();
        let next_chunk_cap = arena.get_capacity() * 2;

        let raw = unsafe { alloc(layout) };
        let current = match NonNull::new(raw.cast::<Arena<Init>>()) {
            Some(ptr) => ptr,
            None => handle_alloc_error(layout),
        };

        unsafe {
            current.as_ptr().write(arena);
        }

        Self {
            current,
            next_chunk_cap,
        }
    }

    pub fn with_layout(layout: Layout) -> Self {
        let arena = Arena::with_layout(layout, None);
        let layout = Layout::new::<Arena<Init>>();
        let next_chunk_cap = arena.get_capacity() * 2;

        let raw = unsafe { alloc(layout) };
        let current = match NonNull::new(raw.cast::<Arena<Init>>()) {
            Some(ptr) => ptr,
            None => handle_alloc_error(layout),
        };

        unsafe {
            current.as_ptr().write(arena);
        }

        Self {
            current,
            next_chunk_cap,
        }
    }

    // pub fn reset(&self) {
    //     let mut cur: Option<NonNull<Arena<Init>>> = Some(self.current);
    //     while let Some(node) = cur {
    //         unsafe {
    //             let ptr: *mut Arena<Init> = node.as_ptr();
    //             let prev = (*ptr).prev.map(|p| p.cast::<Arena<Init>>());
    //             (*ptr).dealloc();
    //             cur = prev;
    //         }
    //     }
    // }
}

impl Drop for Arenas {
    fn drop(&mut self) {
        let mut cur: Option<NonNull<Arena<Init>>> = Some(self.current);
        while let Some(node) = cur {
            unsafe {
                let ptr: *mut Arena<Init> = node.as_ptr();
                let prev = (*ptr).prev.map(|p| p.cast::<Arena<Init>>());
                (*ptr).dealloc();
                cur = prev;
            }
        }
    }
}
