#![allow(dead_code)]

use criterion::Criterion;

mod cpu;

pub fn bench1(c: &mut Criterion) {
    let mut group = c.benchmark_group("sse2 load");
    group.throughput(criterion::Throughput::Elements(96000000));
    group.bench_function("sse2", |b| {
        b.iter(|| cpu::load_select(1000000));
    });

    group.bench_function("sse2_rust", |b| {
        b.iter(|| cpu::rust_load_select(1000000));
    });
}

fn benches() {
    let mut criterion = crate::Criterion::default().configure_from_args();
    bench1(&mut criterion);
}

fn main() {
    benches();
    crate::Criterion::default()
        .configure_from_args()
        .final_summary();
}
