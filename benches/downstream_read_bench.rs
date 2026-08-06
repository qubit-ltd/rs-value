// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Benchmarks read paths exercised by configuration and metadata consumers.

use std::collections::HashMap;
use std::hint::black_box;

use criterion::{
    BatchSize,
    Criterion,
    criterion_group,
    criterion_main,
};
use qubit_datatype::{
    CollectionConversionOptions,
    DataConversionOptions,
    NumericComparisonPolicy,
};
use qubit_value::{
    Value,
    ValueContainer,
    ValueWireRefV1,
    ValueWireV1,
    WireLimits,
};
use serde_json::json;

/// Builds the scalar-string splitting policy used by configuration readers.
fn config_conversion_options() -> DataConversionOptions {
    DataConversionOptions::default().with_collection_options(
        CollectionConversionOptions::default()
            .with_split_scalar_strings(true)
            .with_delimiters([',']),
    )
}

/// Benchmarks scalar and list conversions used by `qubit-config`.
fn benchmark_config_conversions(c: &mut Criterion) {
    let scalar = ValueContainer::from("65535");
    let ports = ValueContainer::from("8080,8081,8082,8083");
    let options = config_conversion_options();

    c.bench_function("downstream/config_scalar_string_to_u32", |bencher| {
        bencher.iter(|| {
            let value = scalar
                .to_first_with::<u32>(black_box(&options))
                .expect("numeric configuration text should convert");
            black_box(value)
        });
    });
    c.bench_function(
        "downstream/config_scalar_string_to_u16_list",
        |bencher| {
            bencher.iter(|| {
                let values = ports
                    .to_list_with::<u16>(black_box(&options))
                    .expect("delimited port text should convert");
                black_box(values)
            });
        },
    );
}

/// Benchmarks mixed-width numeric comparison used by `qubit-metadata`.
fn benchmark_metadata_numeric_comparison(c: &mut Criterion) {
    let left = Value::Int64(i64::MAX);
    let right = Value::UInt64(i64::MAX as u64 + 1);
    let policy = NumericComparisonPolicy::default();

    c.bench_function(
        "downstream/metadata_mixed_integer_comparison",
        |bencher| {
            bencher.iter(|| {
                let ordering = left
                    .numeric_cmp(black_box(&right), black_box(policy))
                    .expect("finite integer values should compare");
                black_box(ordering)
            });
        },
    );
}

/// Benchmarks natural JSON projection used by configuration serialization.
fn benchmark_natural_json_projection(c: &mut Criterion) {
    let values = ValueContainer::from(vec![
        "api".to_string(),
        "worker".to_string(),
        "scheduler".to_string(),
    ]);
    let options = DataConversionOptions::default();

    c.bench_function("downstream/value_container_to_natural_json", |bencher| {
        bencher.iter(|| {
            let json = values
                .to_json_value_with(black_box(&options))
                .expect("string collection should project to JSON");
            black_box(json)
        });
    });
}

/// Benchmarks the V1 wire encoding and bounded decoding paths.
fn benchmark_value_wire_v1(c: &mut Criterion) {
    let wire = ValueWireV1::try_from(ValueContainer::from(vec![
        "api".to_string(),
        "worker".to_string(),
        "scheduler".to_string(),
    ]))
    .expect("construct V1 wire");
    let encoded = serde_json::to_vec(&wire)
        .expect("benchmark wire value should serialize");

    c.bench_function("downstream/value_wire_v1_encode_json", |bencher| {
        bencher.iter(|| {
            let bytes = serde_json::to_vec(black_box(&wire))
                .expect("benchmark wire value should serialize");
            black_box(bytes)
        });
    });
    c.bench_function("downstream/value_wire_v1_decode_json", |bencher| {
        bencher.iter(|| {
            let value = ValueWireV1::decode_json_slice(black_box(&encoded))
                .expect("benchmark wire value should decode");
            black_box(value)
        });
    });

    let borrowed_values = ValueContainer::from(vec![
        "api".to_string(),
        "worker".to_string(),
        "scheduler".to_string(),
    ]);
    c.bench_function(
        "downstream/value_wire_ref_v1_construct_and_encode_json",
        |bencher| {
            bencher.iter(|| {
                let wire =
                    ValueWireRefV1::try_from(black_box(&borrowed_values))
                        .expect("benchmark wire value should validate");
                let bytes = serde_json::to_vec(black_box(&wire))
                    .expect("benchmark wire value should serialize");
                black_box(bytes)
            });
        },
    );

    let borrowed_float_values = ValueContainer::from(
        (0..256)
            .map(|index| index as f64 / 10.0)
            .collect::<Vec<_>>(),
    );
    let borrowed_float_wire = ValueWireRefV1::try_from(&borrowed_float_values)
        .expect("finite benchmark floats should validate");
    c.bench_function(
        "downstream/value_wire_ref_v1_float_encode_json",
        |bencher| {
            bencher.iter(|| {
                let bytes = serde_json::to_vec(black_box(&borrowed_float_wire))
                    .expect("benchmark float wire should serialize");
                black_box(bytes)
            });
        },
    );
    c.bench_function(
        "downstream/value_wire_ref_v1_float_construct_and_encode_json",
        |bencher| {
            bencher.iter(|| {
                let wire =
                    ValueWireRefV1::try_from(black_box(&borrowed_float_values))
                        .expect("benchmark float wire should validate");
                let bytes = serde_json::to_vec(black_box(&wire))
                    .expect("benchmark float wire should serialize");
                black_box(bytes)
            });
        },
    );
}

/// Benchmarks allocation-free semantic budget accounting on numeric storage.
fn benchmark_numeric_wire_budget(c: &mut Criterion) {
    let values = ValueContainer::from((0..4_096_i64).collect::<Vec<_>>());
    let limits = WireLimits::new(0)
        .with_max_nodes(4_097)
        .with_max_collection_items(4_096);

    c.bench_function("downstream/wire_budget_numeric_4096", |bencher| {
        bencher.iter_batched(
            || limits.begin(0).expect("empty input should fit"),
            |mut budget| {
                budget
                    .check_container(black_box(&values))
                    .expect("numeric collection should fit the budget");
                black_box(budget)
            },
            BatchSize::SmallInput,
        )
    });
}

/// Benchmarks allocation-free accounting of a large string map.
fn benchmark_string_map_wire_budget(c: &mut Criterion) {
    let map = (0..1_024)
        .map(|index| (format!("key-{index}"), format!("value-{index}")))
        .collect::<HashMap<_, _>>();
    let value = ValueContainer::Scalar(Value::StringMap(map));
    let limits = WireLimits::new(0)
        .with_max_nodes(1_025)
        .with_max_map_entries(1_024)
        .with_max_string_bytes(32);

    c.bench_function("downstream/wire_budget_string_map_1024", |bencher| {
        bencher.iter_batched(
            || limits.begin(0).expect("empty input should fit"),
            |mut budget| {
                budget
                    .check_container(black_box(&value))
                    .expect("string map should fit the budget");
                black_box(budget)
            },
            BatchSize::SmallInput,
        )
    });
}

/// Benchmarks recursive accounting of a nested JSON value.
fn benchmark_nested_json_wire_budget(c: &mut Criterion) {
    let value = ValueContainer::Scalar(Value::Json(json!({
        "services": (0..64)
            .map(|index| json!({
                "name": format!("service-{index}"),
                "ports": [index, index + 1, index + 2],
            }))
            .collect::<Vec<_>>(),
    })));
    let limits = WireLimits::new(0)
        .with_max_nodes(1_000)
        .with_max_collection_items(64)
        .with_max_map_entries(4)
        .with_max_string_bytes(32)
        .with_max_numeric_bytes(4);

    c.bench_function("downstream/wire_budget_nested_json", |bencher| {
        bencher.iter_batched(
            || limits.begin(0).expect("empty input should fit"),
            |mut budget| {
                budget
                    .check_container(black_box(&value))
                    .expect("nested JSON should fit the budget");
                black_box(budget)
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    benchmark_config_conversions,
    benchmark_metadata_numeric_comparison,
    benchmark_natural_json_projection,
    benchmark_value_wire_v1,
    benchmark_numeric_wire_budget,
    benchmark_string_map_wire_budget,
    benchmark_nested_json_wire_budget,
);
criterion_main!(benches);
