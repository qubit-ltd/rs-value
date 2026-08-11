// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Benchmarks conversion and wire paths owned by `qubit-value`.

use std::hint::black_box;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_datatype::CollectionConversionPolicy;
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::NumericComparisonPolicy;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireRefV1;
use qubit_value::ValueWireV1;

/// Builds the scalar-string splitting policy used by configuration readers.
fn config_conversion_policy() -> ConversionPolicy {
    ConversionPolicy::default().with_collection_policy(
        CollectionConversionPolicy::default()
            .with_split_scalar_strings(true)
            .with_delimiters([',']),
    )
}

/// Benchmarks scalar and list conversion capabilities.
fn benchmark_config_conversions(c: &mut Criterion) {
    let scalar = ValueContainer::from("65535");
    let ports = ValueContainer::from("8080,8081,8082,8083");
    let policy = config_conversion_policy();
    let limits = ConversionLimits::default();

    c.bench_function("value/config_scalar_string_to_u32", |bencher| {
        bencher.iter(|| {
            let value = scalar
                .to_first_with::<u32>(black_box(&policy), black_box(&limits))
                .expect("numeric configuration text should convert");
            black_box(value)
        });
    });
    c.bench_function("value/config_scalar_string_to_u16_list", |bencher| {
        bencher.iter(|| {
            let values = ports
                .to_list_with::<u16>(black_box(&policy), black_box(&limits))
                .expect("delimited port text should convert");
            black_box(values)
        });
    });
}

/// Benchmarks mixed-width numeric comparison capabilities.
fn benchmark_metadata_numeric_comparison(c: &mut Criterion) {
    let left = Value::Int64(i64::MAX);
    let right = Value::UInt64(i64::MAX as u64 + 1);
    let policy = NumericComparisonPolicy::default();

    c.bench_function("value/metadata_mixed_integer_comparison", |bencher| {
        bencher.iter(|| {
            let ordering = left
                .numeric_cmp(black_box(&right), black_box(policy))
                .expect("finite integer values should compare");
            black_box(ordering)
        });
    });
}

/// Benchmarks natural JSON projection capabilities.
fn benchmark_natural_json_projection(c: &mut Criterion) {
    let values = ValueContainer::from(vec![
        "api".to_string(),
        "worker".to_string(),
        "scheduler".to_string(),
    ]);
    let policy = ConversionPolicy::default();
    let limits = ConversionLimits::default();

    c.bench_function("value/value_container_to_natural_json", |bencher| {
        bencher.iter(|| {
            let json = values
                .to_json_value_with(black_box(&policy), black_box(&limits))
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
    let encoded = serde_json::to_vec(&wire).expect("benchmark wire value should serialize");

    c.bench_function("value/value_wire_v1_encode_json_serde", |bencher| {
        bencher.iter(|| {
            let bytes = serde_json::to_vec(black_box(&wire))
                .expect("benchmark wire value should serialize");
            black_box(bytes)
        });
    });
    c.bench_function("value/value_wire_v1_encode_json_bounded", |bencher| {
        bencher.iter(|| {
            let bytes = wire
                .to_json_vec()
                .expect("bounded wire value should serialize");
            black_box(bytes)
        });
    });
    c.bench_function("value/value_wire_v1_decode_json_bounded", |bencher| {
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
        "value/value_wire_ref_v1_construct_and_encode_json_serde",
        |bencher| {
            bencher.iter(|| {
                let wire = ValueWireRefV1::try_from(black_box(&borrowed_values))
                    .expect("benchmark wire value should validate");
                let bytes = serde_json::to_vec(black_box(&wire))
                    .expect("benchmark wire value should serialize");
                black_box(bytes)
            });
        },
    );
    c.bench_function("value/value_wire_ref_v1_encode_json_bounded", |bencher| {
        bencher.iter(|| {
            let wire = ValueWireRefV1::try_from(black_box(&borrowed_values))
                .expect("benchmark wire value should validate");
            let bytes = wire
                .to_json_vec()
                .expect("bounded wire value should serialize");
            black_box(bytes)
        });
    });

    let borrowed_float_values = ValueContainer::from(
        (0..256)
            .map(|index| index as f64 / 10.0)
            .collect::<Vec<_>>(),
    );
    let borrowed_float_wire = ValueWireRefV1::try_from(&borrowed_float_values)
        .expect("finite benchmark floats should validate");
    c.bench_function(
        "value/value_wire_ref_v1_float_encode_json_serde",
        |bencher| {
            bencher.iter(|| {
                let bytes = serde_json::to_vec(black_box(&borrowed_float_wire))
                    .expect("benchmark float wire should serialize");
                black_box(bytes)
            });
        },
    );
    c.bench_function(
        "value/value_wire_ref_v1_float_construct_and_encode_json_serde",
        |bencher| {
            bencher.iter(|| {
                let wire = ValueWireRefV1::try_from(black_box(&borrowed_float_values))
                    .expect("benchmark float wire should validate");
                let bytes = serde_json::to_vec(black_box(&wire))
                    .expect("benchmark float wire should serialize");
                black_box(bytes)
            });
        },
    );
}

criterion_group!(
    benches,
    benchmark_config_conversions,
    benchmark_metadata_numeric_comparison,
    benchmark_natural_json_projection,
    benchmark_value_wire_v1,
);
criterion_main!(benches);
