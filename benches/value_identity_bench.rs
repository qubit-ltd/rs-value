// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Benchmarks for `Value` identity hashing and hash-map lookup.

use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};

use bigdecimal::BigDecimal;
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use num_bigint::BigInt;
use qubit_value::Value;
use serde_json::json;
use std::hint::black_box;

/// Hashes one value with the standard library's default hasher.
fn hash_value(value: &Value) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Builds the string-map fixture used by identity benchmarks.
fn string_map_fixture() -> Value {
    let entries = (0..100)
        .map(|index| {
            (format!("key-{index:03}"), format!("value-{}", index * 17))
        })
        .collect();
    Value::StringMap(entries)
}

/// Builds the nested JSON fixture used by identity benchmarks.
fn nested_json_fixture() -> Value {
    Value::Json(json!({
        "service": {
            "name": "gateway",
            "ports": [8080, 8443],
            "features": {
                "compression": true,
                "protocols": ["http", "https"],
            },
        },
        "workers": [
            {"name": "api", "weight": 3},
            {"name": "jobs", "weight": 2},
        ],
    }))
}

/// Builds a large-coefficient decimal fixture for scale-sensitive hashing.
fn large_big_decimal_fixture() -> Value {
    let coefficient = BigInt::from(1_u8) << 4096_u32;
    Value::BigDecimal(BigDecimal::new(coefficient, -4096))
}

/// Benchmarks hashing a scalar integer value.
fn benchmark_scalar_hash(c: &mut Criterion) {
    let value = Value::Int64(i64::MAX);
    c.bench_function("identity/hash_scalar_int64", |bencher| {
        bencher.iter(|| black_box(hash_value(black_box(&value))))
    });
}

/// Benchmarks hashing a hundred-entry string map.
fn benchmark_string_map_hash(c: &mut Criterion) {
    let value = string_map_fixture();
    c.bench_function("identity/hash_string_map_100", |bencher| {
        bencher.iter(|| black_box(hash_value(black_box(&value))))
    });
}

/// Benchmarks hashing a nested JSON object.
fn benchmark_json_hash(c: &mut Criterion) {
    let value = nested_json_fixture();
    c.bench_function("identity/hash_nested_json", |bencher| {
        bencher.iter(|| black_box(hash_value(black_box(&value))))
    });
}

/// Benchmarks hashing a large-coefficient BigDecimal.
fn benchmark_big_decimal_hash(c: &mut Criterion) {
    let value = large_big_decimal_fixture();
    c.bench_function(
        "identity/hash_big_decimal_large_coefficient_scale",
        |bencher| bencher.iter(|| black_box(hash_value(black_box(&value)))),
    );
}

/// Benchmarks lookup of heterogeneous values in a hash map.
fn benchmark_hash_map_lookup(c: &mut Criterion) {
    let scalar = Value::Int64(i64::MAX);
    let string_map = string_map_fixture();
    let json = nested_json_fixture();
    let decimal = large_big_decimal_fixture();
    let mut table = HashMap::with_capacity(4);
    table.insert(scalar.clone(), 1_usize);
    table.insert(string_map.clone(), 2_usize);
    table.insert(json.clone(), 3_usize);
    table.insert(decimal.clone(), 4_usize);

    c.bench_function("identity/hash_map_value_lookup", |bencher| {
        bencher.iter(|| {
            let total = table
                .get(black_box(&scalar))
                .copied()
                .unwrap_or_default()
                + table
                    .get(black_box(&string_map))
                    .copied()
                    .unwrap_or_default()
                + table.get(black_box(&json)).copied().unwrap_or_default()
                + table.get(black_box(&decimal)).copied().unwrap_or_default();
            black_box(total)
        })
    });
}

criterion_group!(
    benches,
    benchmark_scalar_hash,
    benchmark_string_map_hash,
    benchmark_json_hash,
    benchmark_big_decimal_hash,
    benchmark_hash_map_lookup,
);
criterion_main!(benches);
