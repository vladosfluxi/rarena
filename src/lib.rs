#![allow(dead_code)]
#![allow(unused_imports)]
#![no_std]
pub mod arena;
pub mod boxed;

extern crate alloc as core_alloc;

use crate::arena::{
    AllocError,
    AllocError::SizeOverflow,
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

use core::cell::Cell;

pub struct Arenas {
    pub current: Cell<NonNull<Arena<Init>>>,
    pub next_chunk_cap: Cell<usize>,
}

impl Arenas {
    pub fn new() -> Self {
        let arena = Arena::new(None);
        let layout = Layout::new::<Arena<Init>>();
        let next_chunk_cap: usize = arena.get_capacity() * 2;

        let raw = unsafe { alloc(layout) };
        let current = match NonNull::new(raw.cast::<Arena<Init>>()) {
            Some(ptr) => ptr,
            None => handle_alloc_error(layout),
        };

        unsafe {
            current.as_ptr().write(arena);
        }

        Self {
            current: Cell::new(current),
            next_chunk_cap: Cell::new(next_chunk_cap),
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
            current: Cell::new(current),
            next_chunk_cap: Cell::new(next_chunk_cap),
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
            current: Cell::new(current),
            next_chunk_cap: Cell::new(next_chunk_cap),
        }
    }

    pub fn alloc<'a, T: Clone>(&self, value: T) -> &'a mut T {
        let current = unsafe { self.current.get().as_ref() };

        match current.alloc(value.clone()) {
            Ok(ptr) => ptr,
            Err(AllocError::SizeOverflow) => {
                panic!("Size overflow");
            }
            Err(AllocError::OutOfMemory) => {
                self.add_arena();
                unsafe { self.current.get().as_ref().alloc(value).unwrap() }
            }
        }
    }

    fn add_arena(&self) {
        let capacity = self.next_chunk_cap.get();
        let arena = Arena::with_capacity(capacity, Some(self.current.get()));
        let layout = Layout::new::<Arena<Init>>();

        let raw = unsafe { alloc(layout) };

        let ptr = match NonNull::new(raw.cast::<Arena<Init>>()) {
            Some(ptr) => ptr,
            None => handle_alloc_error(layout),
        };

        unsafe {
            ptr.as_ptr().write(arena);
        }

        self.current.set(ptr);
        self.next_chunk_cap.set(capacity * 2);
    }
}

impl Drop for Arenas {
    fn drop(&mut self) {
        let mut cur: Option<NonNull<Arena<Init>>> = Some(self.current.get());
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
