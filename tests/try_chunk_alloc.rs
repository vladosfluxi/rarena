use core::{
    alloc::Layout,
    cell::Cell,
    ptr::NonNull,
};
use rarena::{
    Arenas,
    arena::{
        AllocError,
        Arena,
        Init,
        Uninit,
    },
};
use std::alloc::{
    alloc,
    handle_alloc_error,
};

#[test]
fn test_smt() {
    let _arenas = Arenas::new();
}

#[test]
fn test_new_sets_current_and_doubles_capacity() {
    let base = Arena::new(None);
    let expected_next_cap = base.get_capacity() * 2;
    drop(base);

    let arenas = Arenas::new();
    let current_ptr = arenas.current.get();
    assert_ne!(current_ptr.as_ptr(), core::ptr::null_mut());
    assert_eq!(arenas.next_chunk_cap.get(), expected_next_cap);

    let stored_cap = unsafe { current_ptr.as_ref().get_capacity() };
    assert_eq!(arenas.next_chunk_cap.get(), stored_cap * 2);
}

#[test]
fn test_with_capacity_sets_current_and_next_chunk_cap() {
    let cap = 512;
    let base = Arena::with_capacity(cap, None);
    let expected_next_cap = base.get_capacity() * 2;
    drop(base);

    let arenas = Arenas::with_capacity(cap);
    assert_ne!(arenas.current.get().as_ptr(), core::ptr::null_mut());
    assert_eq!(arenas.next_chunk_cap.get(), expected_next_cap);
}

#[test]
fn test_with_layout_sets_current_and_next_chunk_cap() {
    let chunk_layout = Layout::from_size_align(1024, 16).unwrap();
    let base = Arena::with_layout(chunk_layout, None);
    let expected_next_cap = base.get_capacity() * 2;
    drop(base);

    let arenas = Arenas::with_layout(chunk_layout);
    assert_ne!(arenas.current.get().as_ptr(), core::ptr::null_mut());
    assert_eq!(arenas.next_chunk_cap.get(), expected_next_cap);
}

#[test]
fn test_current_points_to_distinct_heap_allocation() {
    let arenas = Arenas::new();
    let chunk_ptr = arenas.current.get().as_ptr();
    let stack_addr = &arenas as *const _ as usize;

    assert_ne!(chunk_ptr as usize, stack_addr);
}

#[test]
fn test_drop_single_chunk() {
    let arenas = Arenas::new();
    drop(arenas);
}

#[test]
fn test_drop_linked_chain_of_chunks() {
    let first = Arenas::with_capacity(128);
    let second = Arena::with_capacity(256, Some(first.current.get()));

    let second_layout = Layout::new::<Arena<Init>>();
    let second_raw = unsafe { alloc(second_layout) };
    let second_ptr = NonNull::new(second_raw.cast::<Arena<Init>>())
        .unwrap_or_else(|| handle_alloc_error(second_layout));
    unsafe {
        second_ptr.as_ptr().write(second);
    }

    let third = Arena::with_capacity(512, Some(second_ptr));
    let third_layout = Layout::new::<Arena<Init>>();
    let third_raw = unsafe { alloc(third_layout) };
    let third_ptr = NonNull::new(third_raw.cast::<Arena<Init>>())
        .unwrap_or_else(|| handle_alloc_error(third_layout));
    unsafe {
        third_ptr.as_ptr().write(third);
    }

    core::mem::forget(first);
    let chain = Arenas {
        current: Cell::new(third_ptr),
        next_chunk_cap: Cell::new(1024_usize),
    };

    drop(chain);
}

#[test]
fn test_multiple_controllers_have_distinct_allocations() {
    let controllers: Vec<Arenas> = (0..8)
        .map(|i| Arenas::with_capacity(64 * (i + 1)))
        .collect();
    let ptrs: Vec<usize> = controllers
        .iter()
        .map(|a| a.current.get().as_ptr() as usize)
        .collect();

    for i in 0..ptrs.len() {
        for j in (i + 1)..ptrs.len() {
            assert_ne!(ptrs[i], ptrs[j]);
        }
    }

    drop(controllers);
}
