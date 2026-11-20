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
    state: PhantomData<S>,
}

impl Arena<Init> {
    pub fn get_capacity(&self) -> usize {
        self.cap
    }

    pub fn get_offset(&self) -> usize {
        self.offset.get()
    }

    fn calculate_totalneeded_destaddr(&self, size: usize, align: usize) -> (usize, usize) {
        let current_offset = self.offset.get();
        let current_addr = unsafe { self.ptr_buffer.add(current_offset) as usize };

        let padding = (align - (current_addr % align)) % align;
        let total_needed = padding + size;

        let dest_addr = current_addr + padding;
        (total_needed, dest_addr)
    }

    pub fn new() -> Self {
        Self::with_capacity(4096)
    }

    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::array::<u8>(cap).unwrap();
        Self::with_layout(layout)
    }

    pub fn with_layout(layout: Layout) -> Self {
        let cap = layout.size();
        let raw_ptr = unsafe { alloc(layout) };

        Self {
            offset: Cell::new(0),
            cap,
            ptr_buffer: raw_ptr,
            layout,
            state: PhantomData,
        }
    }

    pub fn alloc_slice<'a, T>(&'a self, count: usize) -> Result<&'a mut [T], &'static str> {
        let align = align_of::<T>();
        let size = size_of::<T>().checked_mul(count).ok_or("Size overflow")?;
        let offset = self.offset.get();
        let (total_needed, dest_addr) = self.calculate_totalneeded_destaddr(size, align);

        let dest_ptr = dest_addr as *mut T;

        if total_needed + offset > self.cap {
            return Err("Not enough memory");
        }

        let fat_ptr: *mut [T] = unsafe { slice::from_raw_parts_mut(dest_ptr, count) };

        self.offset.set(offset + total_needed);

        unsafe { Ok(&mut *fat_ptr) }
    }

    pub fn alloc<'a, T>(&'a self, value: T) -> Result<&'a mut T, &'static str> {
        let size = size_of::<T>();
        let align = align_of::<T>();
        let current_offset = self.offset.get();
        let (total_needed, dest_addr) = self.calculate_totalneeded_destaddr(size, align);
        let dest_ptr = dest_addr as *mut T;

        if total_needed + current_offset > self.cap {
            return Err("Not enough memory");
        }

        unsafe {
            ptr::write(dest_ptr, value);
        }

        self.offset.set(current_offset + total_needed);

        unsafe { Ok(&mut *dest_ptr) }
    }

    pub fn reset(&mut self) {
        self.offset.set(0);
    }

    pub fn allocated_bytes(&self) -> usize {
        self.offset.get()
    }

    pub fn remaining_capacity(&self) -> usize {
        self.cap - self.offset.get()
    }
}
impl<S> Drop for Arena<S> {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr_buffer, self.layout);
        }
    }
}
