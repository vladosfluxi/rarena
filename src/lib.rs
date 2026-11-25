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

use core::mem;

use core::cell::Cell;

pub struct Arenas {
    pub current: Cell<NonNull<Arena<Init>>>,
    pub next_chunk_cap: Cell<usize>,
}

impl Arenas {
    pub fn new() -> Self {
        Self::with_capacity(4096)
    }

    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::array::<u8>(cap).unwrap();
        Self::with_layout(layout)
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
            current
                .as_ptr()
                .write(arena);
        }

        Self {
            current: Cell::new(current),
            next_chunk_cap: Cell::new(next_chunk_cap),
        }
    }

    pub fn alloc<'a, T>(&self, value: T) -> &'a mut T {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        let mut current = unsafe {
            self.current
                .get()
                .as_ref()
        };

        if current
            .try_alloc(size, align)
            .is_err()
        {
            let cap = self
                .next_chunk_cap
                .get();
            let next_size = Self::calculate_next_chunk_size(cap, size, align);

            self.next_chunk_cap
                .set(next_size);
            self.add_arena();

            current = unsafe {
                self.current
                    .get()
                    .as_ref()
            };
        }

        current.alloc(value)
    }

    fn calculate_next_chunk_size(cap: usize, size: usize, align: usize) -> usize {
        let worst_case = (align - 1).saturating_add(size);
        cap.max(worst_case)
    }

    pub fn alloc_slice<'a, T>(&self, count: usize) -> &'a mut [T] {
        let mut current = unsafe {
            self.current
                .get()
                .as_ref()
        };

        let size = mem::size_of::<T>() * count;
        let align = align_of::<T>();

        if current
            .try_alloc(size, align)
            .is_err()
        {
            let cap = self
                .next_chunk_cap
                .get();
            let next_size = Self::calculate_next_chunk_size(cap, size, align);

            self.next_chunk_cap
                .set(next_size);
            self.add_arena();

            current = unsafe {
                self.current
                    .get()
                    .as_ref()
            };
        }

        current.alloc_slice(count)
    }

    fn add_arena(&self) {
        let capacity = self
            .next_chunk_cap
            .get();
        let arena = Arena::with_capacity(
            capacity,
            Some(
                self.current
                    .get(),
            ),
        );
        let layout = Layout::new::<Arena<Init>>();

        let raw = unsafe { alloc(layout) };

        let ptr = match NonNull::new(raw.cast::<Arena<Init>>()) {
            Some(ptr) => ptr,
            None => handle_alloc_error(layout),
        };

        unsafe {
            ptr.as_ptr()
                .write(arena);
        }

        self.current
            .set(ptr);
        self.next_chunk_cap
            .set(capacity.saturating_mul(2));
    }
}

impl Drop for Arenas {
    fn drop(&mut self) {
        let mut cur: Option<NonNull<Arena<Init>>> = Some(
            self.current
                .get(),
        );
        while let Some(node) = cur {
            unsafe {
                let ptr: *mut Arena<Init> = node.as_ptr();
                let prev = (*ptr)
                    .prev
                    .map(|p| p.cast::<Arena<Init>>());
                (*ptr).dealloc();
                cur = prev;
            }
        }
    }
}
