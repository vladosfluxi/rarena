use core::hint::black_box;
use criterion::*;
use rarena::{
    Arenas,
    arena::Arena,
};

#[derive(Default, Clone)]
struct Small(u8);

#[derive(Default, Clone)]
struct Big([usize; 30]);

const ALLOCATIONS: usize = 10_000;

fn alloc_multi<T: Default + Clone>(n: usize, initial_cap: usize) {
    let arenas = Arenas::with_capacity(initial_cap);

    for _ in 0..n {
        let val: &mut T = arenas.alloc(black_box(T::default()));
        black_box(val);
    }
}

fn alloc_single<T: Default>(n: usize) {
    let cap = n * (core::mem::size_of::<T>() + core::mem::align_of::<T>());
    let arena = Arena::with_capacity(cap, None);

    for _ in 0..n {
        let val: &mut T = arena.alloc(black_box(T::default())).unwrap();
        black_box(val);
    }
}

fn alloc_system<T: Default>(n: usize) {
    let mut storage = Vec::with_capacity(n);

    for _ in 0..n {
        let boxed = Box::new(black_box(T::default()));
        storage.push(black_box(boxed));
    }
}

fn bench_alloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("Alloc");
    group.throughput(Throughput::Elements(ALLOCATIONS as u64));

    // Multi-chunk Arenas benchmarks (with small initial cap to trigger growth)
    group.bench_function("small_arenas_grow", |b| {
        b.iter(|| alloc_multi::<Small>(ALLOCATIONS, 64));
    });
    group.bench_function("big_arenas_grow", |b| {
        b.iter(|| alloc_multi::<Big>(ALLOCATIONS, 512));
    });

    // Single-chunk Arena benchmarks (pre-sized)
    group.bench_function("small_arena", |b| {
        b.iter(|| alloc_single::<Small>(ALLOCATIONS));
    });
    group.bench_function("big_arena", |b| {
        b.iter(|| alloc_single::<Big>(ALLOCATIONS));
    });

    // System Box::new comparison benchmarks
    group.bench_function("small_system_box", |b| {
        b.iter(|| alloc_system::<Small>(ALLOCATIONS));
    });
    group.bench_function("big_system_box", |b| {
        b.iter(|| alloc_system::<Big>(ALLOCATIONS));
    });

    group.finish();
}

criterion_group!(benches, bench_alloc);
criterion_main!(benches);
