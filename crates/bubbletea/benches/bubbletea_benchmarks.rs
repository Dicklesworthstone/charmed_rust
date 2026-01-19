//! Placeholder benchmarks for bubbletea.
//!
//! TODO: Add actual benchmarks.

use criterion::{criterion_group, criterion_main, Criterion};

fn placeholder_benchmark(c: &mut Criterion) {
    c.bench_function("placeholder", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, placeholder_benchmark);
criterion_main!(benches);
