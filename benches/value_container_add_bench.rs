// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Benchmarks scalar append paths used by downstream configuration crates.

use std::hint::black_box;

use criterion::BatchSize;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_value::ValueContainer;

/// Benchmarks appending a scalar to a collection with spare capacity.
fn benchmark_append_scalar_to_collection(c: &mut Criterion) {
    c.bench_function("value_container/add_scalar_to_collection", |bencher| {
        bencher.iter_batched(
            || {
                let mut values = Vec::with_capacity(2);
                values.push(1_i32);
                ValueContainer::from(values)
            },
            |mut container| {
                container
                    .add(black_box(2_i32))
                    .expect("matching scalar should append");
                black_box(container)
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmarks promoting scalar storage when another scalar is appended.
fn benchmark_promote_scalar_with_scalar(c: &mut Criterion) {
    c.bench_function("value_container/promote_scalar_with_scalar", |bencher| {
        bencher.iter_batched(
            || ValueContainer::from(1_i32),
            |mut container| {
                container
                    .add(black_box(2_i32))
                    .expect("matching scalar should append");
                black_box(container)
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    benchmark_append_scalar_to_collection,
    benchmark_promote_scalar_with_scalar,
);
criterion_main!(benches);
