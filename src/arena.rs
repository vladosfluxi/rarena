#![allow(dead_code)]
#![allow(unused_imports)]
extern crate alloc as core_alloc;

use core::ptr::{
    self,
    NonNull,
    write,
    write_bytes,
};

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
    handle_alloc_error
};

pub struct Arena {
    offset: Cell<usize>,
    cap: usize,
    ptr_buffer: *mut u8,
    layout:Layout
}

impl Arena {
    // HELPER FUNCTIONS
    pub fn get_capacity(&self) -> usize {
        self.cap
    }

    pub fn get_offset(&self) -> usize {
        self.offset.get()
    }

    pub fn new() -> Self {
        let cap: usize = 4096;
        let layout = Layout::array::<u8>(cap).unwrap();

        let raw_ptr = unsafe { alloc(layout) };

        // if raw_ptr.is_null(){
        //     handle_alloc_error(layout);
        // }

        Self {
            offset: Cell::new(0),
            cap,
            ptr_buffer: raw_ptr,
            layout
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::array::<u8>(cap).unwrap();

        let raw_ptr = unsafe { alloc(layout) };

        

        Self {
            offset: Cell::new(0),
            cap,
            ptr_buffer: raw_ptr,
            layout
        }
    }

    pub fn with_layout(layout:Layout) -> Self{
        let cap = layout.size();
        let raw_ptr = unsafe  {
            alloc(layout)
        };  

        // if raw_ptr.is_null() {
        //     handle_alloc_error(layout);
        // }

        Self {
            offset: Cell::new(0),
            cap,
            ptr_buffer: raw_ptr,
            layout
        }
    }
    
    pub fn alloc_arr<'a, T:Clone>(&self, values: &[T]) -> Result<&'a mut [T], &'static str> {

        let size = size_of_val(&values);
        let align = align_of_val(&values);

        let current_offset = self.offset.get();
        let current_addr = unsafe {
            self.ptr_buffer.add(current_offset) as usize 
        };

        let padding = (align - (current_addr % align)) % align;
        let total_needed = padding + size;

        let dest_addr = current_addr + padding;
        let dest_ptr = dest_addr as *mut T;


        if total_needed + current_offset > self.cap {
            return Err("Not enough memory");
        }
        
        let count = values.len();
        let fat_ptr:*mut [T] = unsafe {slice::from_raw_parts_mut(dest_ptr,count) };

        unsafe {
            for (i,item) in values.iter().enumerate() {
                ptr::write(dest_ptr.add(i),item.clone());
            }
        }

        self.offset.set(current_offset + total_needed) ;
        
        unsafe {
            Ok(&mut *fat_ptr)
        }
    }





    pub fn alloc<'a, T>(&self, value: T) -> Result<&'a mut T, &'static str> {
        let size = size_of::<T>();
        let align = align_of::<T>();

        let current_offset = self.offset.get();
        let current_addr = unsafe { self.ptr_buffer.add(current_offset) as usize };
        let padding = (align - (current_addr % align)) % align;

        let total_needed = padding + size;
        let dest_addr = current_addr + padding;
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

    pub fn reset(&self) {
        self.offset.set(0);
    }

    pub fn allocated_bytes(&self) -> usize {
        self.offset.get()
    }

    pub fn remaining_capacity(&self) -> usize {
        self.cap - self.offset.get()
    }
    
     
}

impl Drop for Arena {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr_buffer,self.layout );
        }
    }
}
