extern crate alloc as core_alloc;

use core::ptr::{
    self,
    NonNull,
    write,
    write_bytes,
};

use core::marker::PhantomData;

use core::cell::Cell;

use core::mem::{
    align_of,
    size_of,
};

use core::slice;

use core::mem::MaybeUninit;
use core_alloc::alloc::{
    Layout,
    LayoutError,
    alloc,
    dealloc,
    handle_alloc_error,
};

pub struct Uninit;
pub struct Init;

pub struct Arena<S = Uninit> {
    offset: Cell<usize>,
    cap: usize,
    ptr_buffer: *mut u8,
    layout: Layout,
    pub prev: Option<NonNull<Arena<Init>>>,
    state: PhantomData<S>,
}

impl Arena<Init> {
    pub fn get_capacity(&self) -> usize {
        self.cap
    }

    pub fn get_offset(&self) -> usize {
        self.offset
            .get()
    }

    fn calculate_totalneeded_destaddr(&self, size: usize, align: usize) -> (usize, usize) {
        let current_offset = self
            .offset
            .get();
        let current_addr = unsafe {
            self.ptr_buffer
                .add(current_offset) as usize
        };

        let padding = (align - (current_addr % align)) % align;
        let total_needed = padding + size;

        let dest_addr = current_addr + padding;
        (total_needed, dest_addr)
    }

    pub fn new(prev: Option<NonNull<Arena<Init>>>) -> Self {
        Self::with_capacity(4096, prev)
    }

    pub fn with_capacity(cap: usize, prev: Option<NonNull<Arena<Init>>>) -> Self {
        let layout = Layout::array::<u8>(cap).unwrap();
        Self::with_layout(layout, prev)
    }

    pub fn with_layout(layout: Layout, prev: Option<NonNull<Arena<Init>>>) -> Self {
        let cap = layout.size();
        let raw_ptr = unsafe { alloc(layout) };

        if raw_ptr.is_null() {
            handle_alloc_error(layout)
        }

        Self {
            offset: Cell::new(0),
            cap,
            ptr_buffer: raw_ptr,
            layout,
            prev,
            state: PhantomData,
        }
    }

    pub fn alloc_slice<'a, T>(&'a self, count: usize) -> &'a mut [T] {
        let align = align_of::<T>();
        let size = size_of::<T>() * count;
        let offset = self
            .offset
            .get();
        let (total_needed, dest_addr) = self.calculate_totalneeded_destaddr(size, align);

        let dest_ptr = dest_addr as *mut T;

        self.offset
            .set(offset + total_needed);

        unsafe { slice::from_raw_parts_mut(dest_ptr, count) }
    }

    pub fn try_alloc(&self, size: usize, align: usize) -> Result<(), AllocError> {
        let cap = self.cap;
        let offset = self
            .offset
            .get();

        let (total_needed, _) = self.calculate_totalneeded_destaddr(size, align);

        if total_needed + offset > cap {
            return Err(AllocError::OutOfMemory);
        }

        Ok(())
    }

    pub fn alloc<'a, T>(&'a self, value: T) -> &'a mut T {
        let size = size_of::<T>();
        let align = align_of::<T>();
        let current_offset = self
            .offset
            .get();
        let (total_needed, dest_addr) = self.calculate_totalneeded_destaddr(size, align);
        let dest_ptr = dest_addr as *mut T;

        unsafe {
            ptr::write(dest_ptr, value);
        }

        self.offset
            .set(current_offset + total_needed);

        unsafe { &mut *dest_ptr }
    }

    pub fn dealloc(&self) {
        unsafe {
            dealloc(self.ptr_buffer, self.layout);
        }
    }

    pub fn reset(&mut self) {
        self.offset
            .set(0);
    }

    pub fn allocated_bytes(&self) -> usize {
        self.offset
            .get()
    }

    pub fn remaining_capacity(&self) -> usize {
        self.cap
            - self
                .offset
                .get()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocError {
    // AllocErr,
    OutOfMemory,
    SizeOverflow,
}
