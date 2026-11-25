use core::{
    hint::black_box,
    time::Duration,
};
use criterion::*;
use rarena::{
    Arenas,
    arena::Arena,
};

#[derive(Default, Clone)]
struct Small(u8);

#[derive(Default, Clone)]
struct Big([usize; 30]);

const ALLOCATIONS: usize = 2_000;

fn alloc_multi<T: Default + Clone>(n: usize, initial_cap: usize) {
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

fn alloc_system<T: Default>(n: usize) {
    let mut storage = Vec::with_capacity(n);

    for _ in 0..n {
        let boxed = Box::new(black_box(T::default()));
        storage.push(black_box(boxed));
    }

    black_box(&storage);
}

fn bench_alloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("Alloc");
    group.throughput(Throughput::Elements(ALLOCATIONS as u64));
    group.sample_size(20);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(2));

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
