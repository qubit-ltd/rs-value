// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Benchmarks conversion and wire paths owned by `qubit-value`.

use std::collections::HashMap;
use std::hint::black_box;
use std::time::Duration;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_budget::json::JsonEncodeLimits;
use qubit_datatype::CollectionConversionPolicy;
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::NumericComparisonPolicy;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWireRefV1;
use qubit_value::ValueWireV1;
use serde_json::Value as JsonValue;
use serde_json::json;
use serde_json::to_vec;

/// Builds the scalar-string splitting policy used by configuration readers.
fn config_conversion_policy() -> ConversionPolicy {
    ConversionPolicy::builder()
        .collection_policy(
            CollectionConversionPolicy::builder()
                .split_scalar_strings(true)
                .delimiters([','])
                .build(),
        )
        .build()
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

    for size in [16, 65_536] {
        let text = Value::from("x".repeat(size));
        assert_eq!(text.to_json_value().unwrap().as_str().unwrap().len(), size);
        c.bench_function(&format!("projection/string/{size}"), |b| {
            b.iter(|| black_box(black_box(&text).to_json_value().unwrap()));
        });
    }
    let strings = ValueContainer::from(vec!["x".repeat(1024); 32]);
    assert_eq!(
        strings.to_json_value().unwrap().as_array().unwrap().len(),
        32
    );
    c.bench_function("projection/string_collection/32", |b| {
        b.iter(|| black_box(black_box(&strings).to_json_value().unwrap()));
    });
    for size in [16, 1024, 4096] {
        let source = Value::Json(JsonValue::Array(vec![JsonValue::Bool(true); size]));
        assert_eq!(
            source.to_json_value().unwrap().as_array().unwrap().len(),
            size
        );
        c.bench_function(&format!("natural_array/{size}"), |b| {
            b.iter(|| {
                black_box(
                    black_box(&source)
                        .to_json_value()
                        .expect("array should project"),
                )
            });
        });
    }

    let durations = ValueContainer::from(vec![Duration::from_millis(1234); 32]);
    assert_eq!(
        durations.to_json_value().unwrap().as_array().unwrap().len(),
        32
    );
    c.bench_function("projection/duration_collection/32", |b| {
        b.iter(|| black_box(black_box(&durations).to_json_value().unwrap()));
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
    let encoded = to_vec(&wire).expect("benchmark wire value should serialize");

    c.bench_function("value/value_wire_v1_encode_json_serde", |bencher| {
        bencher.iter(|| {
            let bytes = to_vec(black_box(&wire)).expect("benchmark wire value should serialize");
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
                let bytes =
                    to_vec(black_box(&wire)).expect("benchmark wire value should serialize");
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
                let bytes = to_vec(black_box(&borrowed_float_wire))
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
                let bytes =
                    to_vec(black_box(&wire)).expect("benchmark float wire should serialize");
                black_box(bytes)
            });
        },
    );
}

/// Builds a deterministic JSON object whose insertion order can be varied.
fn canonical_object_fixture(size: usize, order: &str, nested: bool) -> Value {
    let mut indexes: Vec<_> = (0..size).collect();
    match order {
        "reverse" => indexes.reverse(),
        "shuffled" => indexes.sort_unstable_by_key(|index| (index * 37 + 11) % size.max(1)),
        _ => {}
    }
    let mut object = serde_json::Map::new();
    for index in indexes {
        let key = format!("key-{index:04}");
        let value = if nested {
            json!({"inner": {"leaf": index, "label": format!("value-{index:04}")}})
        } else {
            json!(index)
        };
        object.insert(key, value);
    }
    Value::Json(JsonValue::Object(object))
}

/// Benchmarks canonical object serialization and natural JSON materialization.
fn benchmark_canonical_objects(c: &mut Criterion) {
    for order in ["ordered", "reverse", "shuffled"] {
        for size in [0, 1, 16, 1024] {
            for nested in [false, true] {
                let value = canonical_object_fixture(size, order, nested);
                let wire = ValueWireRefV1::from_value(&value)
                    .expect("object fixture must be wire encodable");
                let shape = if nested { "nested" } else { "flat" };
                let label = format!("canonical_object/{shape}/{order}/{size}");
                c.bench_function(&format!("{label}/wire"), |b| {
                    b.iter(|| {
                        black_box(to_vec(black_box(&wire)).expect("wire object should serialize"))
                    });
                });
                c.bench_function(&format!("{label}/natural"), |b| {
                    b.iter(|| {
                        black_box(
                            black_box(&value)
                                .to_json_value()
                                .expect("object should project"),
                        )
                    });
                });
            }
        }
    }
    for size in [0, 1, 16, 1024] {
        let map: HashMap<_, _> = (0..size)
            .map(|index| (format!("key-{index:04}"), format!("value-{index:04}")))
            .collect();
        let scalar = Value::StringMap(map.clone());
        let collection = MultiValues::StringMap(vec![map]);
        c.bench_function(&format!("string_map/{size}/natural"), |bencher| {
            bencher.iter(|| {
                black_box(
                    black_box(&scalar)
                        .to_json_value()
                        .expect("string map should project"),
                )
            });
        });
        c.bench_function(&format!("string_map/{size}/collection"), |bencher| {
            bencher.iter(|| {
                black_box(
                    black_box(&collection)
                        .to_json_value()
                        .expect("string map collection should project"),
                )
            });
        });
    }
}

/// Builds the three fixture sizes used to compare preparation and encoding.
fn wire_read_fixture(size: usize) -> ValueContainer {
    ValueContainer::from(
        (0..size)
            .map(|index| format!("item-{index:04}"))
            .collect::<Vec<_>>(),
    )
}

/// Benchmarks the bounded preflight pass without allocating the wire payload.
fn benchmark_preflight(c: &mut Criterion) {
    let limits = JsonEncodeLimits::builder()
        .max_output_bytes(4 * 1024 * 1024)
        .build();
    for size in [1, 32, 4096] {
        let values = wire_read_fixture(size);
        c.bench_function(&format!("preflight/{size}"), |bencher| {
            bencher.iter(|| {
                let mut checker = ValueWireEncodePreflight::new(black_box(limits));
                let result = checker.check_container(black_box(&values));
                black_box(result)
            });
        });
    }
}

/// Benchmarks encoding an already prepared borrowed DTO, including its internal
/// preflight.
fn benchmark_borrowed_encode(c: &mut Criterion) {
    let limits = JsonEncodeLimits::builder()
        .max_output_bytes(4 * 1024 * 1024)
        .build();
    for size in [1, 32, 4096] {
        let values = wire_read_fixture(size);
        let wire = ValueWireRefV1::try_from(&values).expect("fixture must be wire encodable");
        c.bench_function(&format!("borrowed_encode/{size}"), |bencher| {
            bencher.iter(|| {
                let result = black_box(&wire).to_json_vec_with_limits(black_box(limits));
                black_box(result)
            });
        });
    }
}

/// Benchmarks explicit preflight, borrowed DTO construction, internal
/// preflight, and encoding.
fn benchmark_preflight_and_encode(c: &mut Criterion) {
    let limits = JsonEncodeLimits::builder()
        .max_output_bytes(4 * 1024 * 1024)
        .build();
    for size in [1, 32, 4096] {
        let values = wire_read_fixture(size);
        c.bench_function(&format!("preflight_and_encode/{size}"), |bencher| {
            bencher.iter(|| {
                let mut checker = ValueWireEncodePreflight::new(black_box(limits));
                checker
                    .check_container(black_box(&values))
                    .expect("fixture must pass preflight");
                let wire = ValueWireRefV1::try_from(black_box(&values))
                    .expect("fixture must be wire encodable");
                let result = wire.to_json_vec_with_limits(black_box(limits));
                black_box(result)
            });
        });
    }
}

/// Benchmarks the extra borrowed wire DTO validation between preflight and
/// encoding.
fn benchmark_repeated_wire_validation(c: &mut Criterion) {
    let fixtures = [
        (
            "strings",
            ValueContainer::from(vec!["item".to_owned(); 4096]),
        ),
        (
            "floats",
            ValueContainer::from(
                (0..4096)
                    .map(|index| index as f64 / 10.0)
                    .collect::<Vec<_>>(),
            ),
        ),
        (
            "json",
            ValueContainer::from(MultiValues::Json(
                (0..4096)
                    .map(|index| json!({"outer": {"items": [index, index as f64 / 10.0]}}))
                    .collect(),
            )),
        ),
    ];
    let limits = JsonEncodeLimits::builder()
        .max_output_bytes(4 * 1024 * 1024)
        .build();

    for (name, values) in fixtures {
        let single = || {
            let mut preflight = ValueWireEncodePreflight::new(limits);
            preflight
                .check_container(&values)
                .expect("fixture must pass preflight");
            let wire = ValueWireRefV1::try_from(&values).expect("fixture must be wire encodable");
            wire.to_json_vec_with_limits(limits)
                .expect("fixture must encode")
        };
        let repeated = || {
            let mut preflight = ValueWireEncodePreflight::new(limits);
            preflight
                .check_container(&values)
                .expect("fixture must pass preflight");
            let _ = black_box(
                ValueWireRefV1::try_from(black_box(&values))
                    .expect("fixture must be wire encodable"),
            );
            let wire = ValueWireRefV1::try_from(&values).expect("fixture must be wire encodable");
            wire.to_json_vec_with_limits(limits)
                .expect("fixture must encode")
        };
        assert_eq!(
            single(),
            repeated(),
            "single and repeated output must match"
        );

        c.bench_function(&format!("repeated_validation/{name}/single"), |bencher| {
            bencher.iter(|| black_box(single()));
        });
        c.bench_function(&format!("repeated_validation/{name}/repeated"), |bencher| {
            bencher.iter(|| black_box(repeated()));
        });
    }
}

/// Benchmarks borrowed DTO validation for JSON trees containing ordinary
/// floats.
fn benchmark_nested_json_number_validation(c: &mut Criterion) {
    let limits = JsonEncodeLimits::builder()
        .max_output_bytes(4 * 1024 * 1024)
        .build();
    for size in [256, 4096] {
        let value = Value::Json(JsonValue::Array(
            (0..size).map(|index| json!(index as f64 / 10.0)).collect(),
        ));
        let payload =
            ValueWirePayloadRefV1::from_value(&value).expect("JSON float fixture must validate");
        let encoded = payload
            .to_json_vec_with_limits(limits)
            .expect("JSON float fixture must encode within the output limit");
        assert!(!encoded.is_empty());

        c.bench_function(
            &format!("nested_json_number_validation/dto/{size}"),
            |bencher| {
                bencher.iter(|| {
                    let payload = ValueWirePayloadRefV1::from_value(black_box(&value))
                        .expect("JSON float fixture must validate");
                    black_box(payload)
                });
            },
        );
        c.bench_function(
            &format!("nested_json_number_validation/dto_and_encode/{size}"),
            |bencher| {
                bencher.iter(|| {
                    let payload = ValueWirePayloadRefV1::from_value(black_box(&value))
                        .expect("JSON float fixture must validate");
                    let encoded = payload
                        .to_json_vec_with_limits(black_box(limits))
                        .expect("JSON float fixture must encode");
                    black_box(encoded)
                });
            },
        );
    }
}

criterion_group!(
    benches,
    benchmark_config_conversions,
    benchmark_metadata_numeric_comparison,
    benchmark_natural_json_projection,
    benchmark_value_wire_v1,
    benchmark_preflight,
    benchmark_borrowed_encode,
    benchmark_preflight_and_encode,
    benchmark_repeated_wire_validation,
    benchmark_nested_json_number_validation,
    benchmark_canonical_objects,
);
criterion_main!(benches);
