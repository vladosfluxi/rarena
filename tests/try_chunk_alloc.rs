use core::{
    alloc::Layout,
    cell::Cell,
    ptr::NonNull,
};
use rarena::{
    Arenas,
    arena::{
        Arena,
        Init,
    },
};
use std::alloc::{
    alloc,
    handle_alloc_error,
};

#[derive(Debug, PartialEq, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

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
    let current_ptr = arenas
        .current
        .get();
    assert_ne!(current_ptr.as_ptr(), core::ptr::null_mut());
    assert_eq!(
        arenas
            .next_chunk_cap
            .get(),
        expected_next_cap
    );

    let stored_cap = unsafe {
        current_ptr
            .as_ref()
            .get_capacity()
    };
    assert_eq!(
        arenas
            .next_chunk_cap
            .get(),
        stored_cap * 2
    );
}

#[test]
fn test_with_capacity_sets_current_and_next_chunk_cap() {
    let cap = 512;
    let base = Arena::with_capacity(cap, None);
    let expected_next_cap = base.get_capacity() * 2;
    drop(base);

    let arenas = Arenas::with_capacity(cap);
    assert_ne!(
        arenas
            .current
            .get()
            .as_ptr(),
        core::ptr::null_mut()
    );
    assert_eq!(
        arenas
            .next_chunk_cap
            .get(),
        expected_next_cap
    );
}

#[test]
fn test_with_layout_sets_current_and_next_chunk_cap() {
    let chunk_layout = Layout::from_size_align(1024, 16).unwrap();
    let base = Arena::with_layout(chunk_layout, None);
    let expected_next_cap = base.get_capacity() * 2;
    drop(base);

    let arenas = Arenas::with_layout(chunk_layout);
    assert_ne!(
        arenas
            .current
            .get()
            .as_ptr(),
        core::ptr::null_mut()
    );
    assert_eq!(
        arenas
            .next_chunk_cap
            .get(),
        expected_next_cap
    );
}

#[test]
fn test_current_points_to_distinct_heap_allocation() {
    let arenas = Arenas::new();
    let chunk_ptr = arenas
        .current
        .get()
        .as_ptr();
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
    let second = Arena::with_capacity(
        256,
        Some(
            first
                .current
                .get(),
        ),
    );

    let second_layout = Layout::new::<Arena<Init>>();
    let second_raw = unsafe { alloc(second_layout) };
    let second_ptr = NonNull::new(second_raw.cast::<Arena<Init>>())
        .unwrap_or_else(|| handle_alloc_error(second_layout));
    unsafe {
        second_ptr
            .as_ptr()
            .write(second);
    }

    let third = Arena::with_capacity(512, Some(second_ptr));
    let third_layout = Layout::new::<Arena<Init>>();
    let third_raw = unsafe { alloc(third_layout) };
    let third_ptr = NonNull::new(third_raw.cast::<Arena<Init>>())
        .unwrap_or_else(|| handle_alloc_error(third_layout));
    unsafe {
        third_ptr
            .as_ptr()
            .write(third);
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
        .map(|a| {
            a.current
                .get()
                .as_ptr() as usize
        })
        .collect();

    for i in 0..ptrs.len() {
        for j in (i + 1)..ptrs.len() {
            assert_ne!(ptrs[i], ptrs[j]);
        }
    }

    drop(controllers);
}

#[test]
fn test_alloc_primitives_and_mutate() {
    let arenas = Arenas::new();
    let n: &mut i32 = arenas.alloc(10_i32);
    assert_eq!(*n, 10);

    *n = 25;
    assert_eq!(*n, 25);
}

#[test]
fn test_alloc_struct_and_compare() {
    let arenas = Arenas::new();
    let p1: &mut Point = arenas.alloc(Point {
        x: 1,
        y: 2,
    });
    let p2: &mut Point = arenas.alloc(Point {
        x: -5,
        y: 12,
    });

    assert_eq!(
        *p1,
        Point {
            x: 1,
            y: 2
        }
    );
    assert_eq!(
        *p2,
        Point {
            x: -5,
            y: 12
        }
    );

    p2.x = 7;
    assert_eq!(
        *p2,
        Point {
            x: 7,
            y: 12
        }
    );
}

#[test]
fn test_alloc_clone_string() {
    let arenas = Arenas::new();
    let original = String::from("arena item");
    let allocated: &mut String = arenas.alloc(original.clone());

    assert_eq!(original, "arena item");
    assert_eq!(*allocated, "arena item");

    allocated.push_str(" updated");
    assert_eq!(*allocated, "arena item updated");
}

#[test]
fn test_alloc_grows_across_chunks() {
    let arenas = Arenas::with_capacity(core::mem::size_of::<Point>());
    let initial_ptr = arenas
        .current
        .get();

    let first = arenas.alloc(Point {
        x: 1,
        y: 1,
    });
    let second = arenas.alloc(Point {
        x: 2,
        y: 2,
    });
    let third = arenas.alloc(Point {
        x: 3,
        y: 3,
    });

    assert_eq!(
        *first,
        Point {
            x: 1,
            y: 1
        }
    );
    assert_eq!(
        *second,
        Point {
            x: 2,
            y: 2
        }
    );
    assert_eq!(
        *third,
        Point {
            x: 3,
            y: 3
        }
    );
    assert_ne!(
        arenas
            .current
            .get(),
        initial_ptr
    );
}
#[repr(align(64))]
#[derive(Debug, PartialEq, Clone, Copy)]
struct AlignedBlock([u8; 24]);

#[test]
fn test_alloc_slice_init_and_mutate() {
    let arenas = Arenas::with_capacity(128);
    let slice: &mut [i32] = arenas.alloc_slice::<i32>(4);
    assert_eq!(slice.len(), 4);

    for (i, slot) in slice
        .iter_mut()
        .enumerate()
    {
        *slot = (i as i32 + 1) * 10;
    }

    assert_eq!(slice, &[10, 20, 30, 40]);
    slice[1] = -99;
    assert_eq!(slice[1], -99);
}

#[test]
fn test_alloc_slice_structs_init() {
    let arenas = Arenas::new();
    let slice: &mut [Point] = arenas.alloc_slice::<Point>(3);
    assert_eq!(slice.len(), 3);

    for (i, slot) in slice
        .iter_mut()
        .enumerate()
    {
        let v = i as i32;
        *slot = Point {
            x: v,
            y: v * 2,
        };
    }

    assert_eq!(
        slice,
        &[
            Point {
                x: 0,
                y: 0
            },
            Point {
                x: 1,
                y: 2
            },
            Point {
                x: 2,
                y: 4
            },
        ]
    );
}

#[test]
fn test_alloc_slice_zero_len() {
    let arenas = Arenas::with_capacity(64);
    let slice: &mut [Point] = arenas.alloc_slice::<Point>(0);

    assert!(slice.is_empty());
    assert_eq!(slice.len(), 0);
}

#[test]
fn test_alloc_slice_alignment_after_unaligned_alloc() {
    let arenas = Arenas::with_capacity(128);
    let _byte = arenas.alloc(7_u8);

    let slice: &mut [AlignedBlock] = arenas.alloc_slice::<AlignedBlock>(2);
    assert_eq!(slice.len(), 2);
    assert_eq!((slice.as_ptr() as usize) % core::mem::align_of::<AlignedBlock>(), 0);

    slice[0] = AlignedBlock([1; 24]);
    slice[1] = AlignedBlock([2; 24]);
    assert_eq!(slice[0], AlignedBlock([1; 24]));
    assert_eq!(slice[1], AlignedBlock([2; 24]));
}

#[test]
fn test_alloc_slice_grows_across_chunks() {
    let arenas = Arenas::with_capacity(32);
    let initial_ptr = arenas
        .current
        .get();

    for i in 0..32_i32 {
        let slice: &mut [i32] = arenas.alloc_slice::<i32>(16);
        assert_eq!(slice.len(), 16);

        for (j, slot) in slice
            .iter_mut()
            .enumerate()
        {
            *slot = i * 100 + j as i32;
        }

        assert_eq!(slice[0], i * 100);
        assert_eq!(slice[15], i * 100 + 15);
    }

    assert_ne!(
        arenas
            .current
            .get(),
        initial_ptr
    );
}
#[repr(align(64))]
#[derive(Debug, PartialEq, Clone, Copy)]
struct AlignedItem(u64);

#[test]
fn test_alloc_array_primitives_and_mutate() {
    let arenas = Arenas::with_capacity(128);
    let arr: &mut [i32; 4] = arenas.alloc([10, 20, 30, 40]);

    assert_eq!(*arr, [10, 20, 30, 40]);
    arr[0] = -5;
    arr[2] = 77;
    assert_eq!(*arr, [-5, 20, 77, 40]);
}

#[test]
fn test_alloc_array_structs() {
    let arenas = Arenas::new();
    let arr: &mut [Point; 2] = arenas.alloc([
        Point {
            x: 1,
            y: 2,
        },
        Point {
            x: 3,
            y: 4,
        },
    ]);

    assert_eq!(
        arr[0],
        Point {
            x: 1,
            y: 2
        }
    );
    assert_eq!(
        arr[1],
        Point {
            x: 3,
            y: 4
        }
    );

    arr[1].x = 9;
    assert_eq!(
        arr[1],
        Point {
            x: 9,
            y: 4
        }
    );
}

#[test]
fn test_alloc_zero_sized_array() {
    let arenas = Arenas::with_capacity(64);
    let empty: &mut [Point; 0] = arenas.alloc([]);

    assert_eq!(empty.len(), 0);
}

#[test]
fn test_alloc_array_alignment_after_unaligned_alloc() {
    let arenas = Arenas::with_capacity(256);
    let _byte = arenas.alloc(7_u8);

    let arr: &mut [AlignedItem; 3] = arenas.alloc([AlignedItem(1), AlignedItem(2), AlignedItem(3)]);

    assert_eq!((arr.as_ptr() as usize) % core::mem::align_of::<AlignedItem>(), 0);
    assert_eq!(*arr, [AlignedItem(1), AlignedItem(2), AlignedItem(3)]);
}

#[test]
fn test_alloc_array_grows_across_chunks() {
    let arenas = Arenas::with_capacity(32);
    let initial_ptr = arenas
        .current
        .get();

    for i in 0..20_i32 {
        let base = i * 4;
        let arr: &mut [i32; 4] = arenas.alloc([base, base + 1, base + 2, base + 3]);
        assert_eq!(*arr, [base, base + 1, base + 2, base + 3]);
    }

    assert_ne!(
        arenas
            .current
            .get(),
        initial_ptr
    );
}
