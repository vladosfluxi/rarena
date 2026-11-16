use core::hint::black_box;
use criterion::*;
use rarena::arena::Arena;

#[derive(Default)]
struct Small(u8);

#[derive(Default)]
struct Big([usize; 30]);

const ALLOCATIONS: usize = 10_000;

fn alloc<T: Default>(n: usize) {
    let cap = n * (std::mem::size_of::<T>() + std::mem::align_of::<T>());
    let arena = Arena::with_capacity(cap);

    for _ in 0..n {
        let val: &mut T = arena
            .alloc(black_box(Default::default()))
            .unwrap_or_else(|_| unsafe { core::hint::unreachable_unchecked() });

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

    // Arena benchmarks (original)
    group.bench_function("small", |b| b.iter(|| alloc::<Small>(ALLOCATIONS)));
    group.bench_function("big", |b| b.iter(|| alloc::<Big>(ALLOCATIONS)));

    // System Box::new comparison benchmarks
    group.bench_function("small_system_box", |b| b.iter(|| alloc_system::<Small>(ALLOCATIONS)));
    group.bench_function("big_system_box", |b| b.iter(|| alloc_system::<Big>(ALLOCATIONS)));

    group.finish();
}

criterion_group!(benches, bench_alloc);
criterion_main!(benches);
