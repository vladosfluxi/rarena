use core::{
    hint::black_box,
    time::Duration,
};
use criterion::*;
use rarena::{
    Arenas,
    arena::Arena,
};

#[derive(Default, Clone, Copy)]
struct Small(u8);

#[derive(Default, Clone, Copy)]
struct Big([usize; 30]);

const ALLOCATIONS: usize = 256;
const SLICE_LEN: usize = 8;
const ARRAY_LEN: usize = 8;

fn alloc_multi<T: Default>(n: usize, initial_cap: usize) {
    let arenas = Arenas::with_capacity(initial_cap);

    for _ in 0..n {
        let val: &mut T = arenas.alloc(black_box(T::default()));
        black_box(val);
    }
}

fn alloc_single<T: Default>(n: usize) {
    let cap = n
        .saturating_mul(core::mem::size_of::<T>())
        .saturating_add(core::mem::align_of::<T>());
    let arena = Arena::with_capacity(cap, None);

    for _ in 0..n {
        let val: &mut T = arena.alloc(black_box(T::default()));
        black_box(val);
    }
}

fn alloc_slice_multi<T: Default>(n: usize, slice_len: usize, initial_cap: usize) {
    let arenas = Arenas::with_capacity(initial_cap);

    for _ in 0..n {
        let slice: &mut [T] = arenas.alloc_slice::<T>(black_box(slice_len));
        for item in slice.iter_mut() {
            *item = black_box(T::default());
        }
        black_box(slice);
    }
}

fn alloc_slice_single<T: Default>(n: usize, slice_len: usize) {
    let per_slice = slice_len.saturating_mul(core::mem::size_of::<T>());
    let cap = n
        .saturating_mul(per_slice)
        .saturating_add(core::mem::align_of::<T>());
    let arena = Arena::with_capacity(cap, None);

    for _ in 0..n {
        let slice: &mut [T] = arena.alloc_slice::<T>(black_box(slice_len));
        for item in slice.iter_mut() {
            *item = black_box(T::default());
        }
        black_box(slice);
    }
}

fn alloc_array_multi<T: Default, const N: usize>(n: usize, initial_cap: usize) {
    let arenas = Arenas::with_capacity(initial_cap);

    for _ in 0..n {
        let arr: &mut [T; N] = arenas.alloc(black_box(core::array::from_fn(|_| T::default())));
        black_box(arr);
    }
}

fn alloc_array_single<T: Default, const N: usize>(n: usize) {
    let cap = n
        .saturating_mul(core::mem::size_of::<[T; N]>())
        .saturating_add(core::mem::align_of::<[T; N]>());
    let arena = Arena::with_capacity(cap, None);

    for _ in 0..n {
        let arr: &mut [T; N] = arena.alloc(black_box(core::array::from_fn(|_| T::default())));
        black_box(arr);
    }
}

fn bench_alloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("Alloc");
    group.throughput(Throughput::Elements(ALLOCATIONS as u64));
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));

    group.bench_function("small_arenas_grow", |b| {
        b.iter(|| alloc_multi::<Small>(ALLOCATIONS, 64));
    });
    group.bench_function("big_arenas_grow", |b| {
        b.iter(|| alloc_multi::<Big>(ALLOCATIONS, 512));
    });

    group.bench_function("small_arena", |b| {
        b.iter(|| alloc_single::<Small>(ALLOCATIONS));
    });
    group.bench_function("big_arena", |b| {
        b.iter(|| alloc_single::<Big>(ALLOCATIONS));
    });

    group.bench_function("small_slice_arenas_grow", |b| {
        b.iter(|| alloc_slice_multi::<Small>(ALLOCATIONS, SLICE_LEN, 128));
    });
    group.bench_function("big_slice_arenas_grow", |b| {
        b.iter(|| alloc_slice_multi::<Big>(ALLOCATIONS, SLICE_LEN, 4096));
    });

    group.bench_function("small_slice_arena", |b| {
        b.iter(|| alloc_slice_single::<Small>(ALLOCATIONS, SLICE_LEN));
    });
    group.bench_function("big_slice_arena", |b| {
        b.iter(|| alloc_slice_single::<Big>(ALLOCATIONS, SLICE_LEN));
    });

    group.bench_function("small_array_arenas_grow", |b| {
        b.iter(|| alloc_array_multi::<Small, ARRAY_LEN>(ALLOCATIONS, 128));
    });
    group.bench_function("big_array_arenas_grow", |b| {
        b.iter(|| alloc_array_multi::<Big, ARRAY_LEN>(ALLOCATIONS, 4096));
    });

    group.bench_function("small_array_arena", |b| {
        b.iter(|| alloc_array_single::<Small, ARRAY_LEN>(ALLOCATIONS));
    });
    group.bench_function("big_array_arena", |b| {
        b.iter(|| alloc_array_single::<Big, ARRAY_LEN>(ALLOCATIONS));
    });

    group.finish();
}

criterion_group!(benches, bench_alloc);
criterion_main!(benches);
