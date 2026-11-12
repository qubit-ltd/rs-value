// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Allocation regression tests for Wire V1 preflight and payload construction.

#![cfg(all(feature = "big-decimal", feature = "json"))]

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_budget::BudgetError;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::MultiValues;
use qubit_value::NamedMultiValues;
use qubit_value::NamedValue;
use qubit_value::Value;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWireRefV1;
use qubit_value::ValueWireV1;
use serde_json::Number;
use serde_json::from_str;

use self::preflight_allocation_tests::internal::assertions::assert_big_decimal_payload_construction_does_not_copy_coefficient;
use self::preflight_allocation_tests::internal::assertions::assert_bounded_direct_helper_allocations;
use self::preflight_allocation_tests::internal::assertions::assert_expected_encode_limit;
use self::preflight_allocation_tests::internal::assertions::assert_expected_limit;
use self::preflight_allocation_tests::internal::fixtures::json_number_fixture;
use self::preflight_allocation_tests::internal::measurement::measured_allocations;
use self::preflight_allocation_tests::internal::measurement::measured_json_payload_construction_bytes;
use self::preflight_allocation_tests::internal::measurement::measured_preflight_bytes;

// Keeps support modules local to this explicitly registered allocation test
// target.
mod preflight_allocation_tests {
    pub(crate) mod internal;
}

#[test]
fn test_big_decimal_preflight_and_payload_construction_do_not_copy_coefficient() {
    let small = Value::BigDecimal(BigDecimal::new(BigInt::from(123_i32), 2));
    let coefficient = "9".repeat(128 * 1024);
    let large = Value::BigDecimal(BigDecimal::new(
        BigInt::parse_bytes(coefficient.as_bytes(), 10).expect("large coefficient must parse"),
        2,
    ));

    let small_bytes = measured_preflight_bytes(&small);
    let large_bytes = measured_preflight_bytes(&large);

    assert!(
        large_bytes <= small_bytes.saturating_add(4096),
        "preflight allocation grew with the coefficient: small={small_bytes}, large={large_bytes}",
    );

    assert_big_decimal_payload_construction_does_not_copy_coefficient();
}

#[test]
fn test_big_decimal_preflight_rejection_allocation_does_not_scale_with_coefficient() {
    let small_digits = "9".repeat(4096);
    let large_digits = "9".repeat(128 * 1024);
    let small = Value::BigDecimal(BigDecimal::new(
        BigInt::parse_bytes(small_digits.as_bytes(), 10).expect("small coefficient must parse"),
        2,
    ));
    let large = Value::BigDecimal(BigDecimal::new(
        BigInt::parse_bytes(large_digits.as_bytes(), 10).expect("large coefficient must parse"),
        2,
    ));
    let base = ValueWireV1::default_json_encode_limits();
    let profiles = [
        (
            base.into_builder().max_string_bytes(64).build(),
            JsonResource::StringBytes,
        ),
        (
            base.into_builder().max_payload_bytes(128).build(),
            JsonResource::PayloadBytes,
        ),
        (
            base.into_builder().max_output_bytes(256).build(),
            JsonResource::OutputBytes,
        ),
    ];

    for (limits, resource) in profiles {
        let (small_error, small_bytes) =
            measured_allocations(|| ValueWireEncodePreflight::new(limits).check_value(&small));
        let (large_error, large_bytes) =
            measured_allocations(|| ValueWireEncodePreflight::new(limits).check_value(&large));
        assert_expected_limit(
            small_error.expect_err("small fixture must exceed this profile"),
            resource,
        );
        assert_expected_limit(
            large_error.expect_err("large fixture must exceed this profile"),
            resource,
        );
        assert!(
            large_bytes <= small_bytes.saturating_add(4096),
            "rejected coefficient preflight allocation grew: resource={resource:?}, 4096 digits={small_bytes}, 128 Ki digits={large_bytes}",
        );
    }
}

#[test]
fn test_direct_wire_helpers_reject_large_decimal_before_formatting() {
    let small_digits = "9".repeat(4096);
    let large_digits = "9".repeat(128 * 1024);
    let small_scalar = Value::BigDecimal(BigDecimal::new(
        BigInt::parse_bytes(small_digits.as_bytes(), 10).expect("small coefficient must parse"),
        2,
    ));
    let large_scalar = Value::BigDecimal(BigDecimal::new(
        BigInt::parse_bytes(large_digits.as_bytes(), 10).expect("large coefficient must parse"),
        2,
    ));
    let small_collection = MultiValues::BigDecimal(vec![BigDecimal::new(
        BigInt::parse_bytes(small_digits.as_bytes(), 10).expect("small coefficient must parse"),
        2,
    )]);
    let large_collection = MultiValues::BigDecimal(vec![BigDecimal::new(
        BigInt::parse_bytes(large_digits.as_bytes(), 10).expect("large coefficient must parse"),
        2,
    )]);

    let small_payload_ref = ValueWirePayloadRefV1::from_value(&small_scalar).unwrap();
    let large_payload_ref = ValueWirePayloadRefV1::from_value(&large_scalar).unwrap();
    let small_wire_ref = ValueWireRefV1::from_value(&small_scalar).unwrap();
    let large_wire_ref = ValueWireRefV1::from_value(&large_scalar).unwrap();
    let small_collection_payload_ref =
        ValueWirePayloadRefV1::from_values(&small_collection).unwrap();
    let large_collection_payload_ref =
        ValueWirePayloadRefV1::from_values(&large_collection).unwrap();
    let small_collection_wire_ref = ValueWireRefV1::from_values(&small_collection).unwrap();
    let large_collection_wire_ref = ValueWireRefV1::from_values(&large_collection).unwrap();

    // Owned DTO cloning happens outside the measured helper calls.
    let small_payload = ValueWirePayloadV1::try_from(small_scalar.clone()).unwrap();
    let large_payload = ValueWirePayloadV1::try_from(large_scalar.clone()).unwrap();
    let small_wire = ValueWireV1::try_from(small_scalar.clone()).unwrap();
    let large_wire = ValueWireV1::try_from(large_scalar.clone()).unwrap();
    let small_collection_payload = ValueWirePayloadV1::try_from(small_collection.clone()).unwrap();
    let large_collection_payload = ValueWirePayloadV1::try_from(large_collection.clone()).unwrap();
    let small_collection_wire = ValueWireV1::try_from(small_collection.clone()).unwrap();
    let large_collection_wire = ValueWireV1::try_from(large_collection.clone()).unwrap();

    macro_rules! check_helpers {
        ($label:literal, $small:expr, $large:expr) => {{
            let profiles = [
                (
                    JsonEncodeLimits::builder().max_string_bytes(64).build(),
                    JsonResource::StringBytes,
                ),
                (
                    JsonEncodeLimits::builder().max_payload_bytes(128).build(),
                    JsonResource::PayloadBytes,
                ),
                (
                    JsonEncodeLimits::builder().max_output_bytes(256).build(),
                    JsonResource::OutputBytes,
                ),
            ];
            for (limits, resource) in profiles {
                let (small_result, small_bytes) =
                    measured_allocations(|| $small.to_json_vec_with_limits(limits));
                let (large_result, large_bytes) =
                    measured_allocations(|| $large.to_json_vec_with_limits(limits));
                assert_expected_encode_limit(small_result.unwrap_err(), resource, $label, "vector");
                assert_expected_encode_limit(large_result.unwrap_err(), resource, $label, "vector");
                assert_bounded_direct_helper_allocations(
                    $label,
                    "vector",
                    resource,
                    small_bytes,
                    large_bytes,
                );

                let mut small_writer = b"prefix".to_vec();
                let mut large_writer = b"prefix".to_vec();
                let (small_result, small_bytes) = measured_allocations(|| {
                    $small.to_json_writer_with_limits(&mut small_writer, limits)
                });
                let (large_result, large_bytes) = measured_allocations(|| {
                    $large.to_json_writer_with_limits(&mut large_writer, limits)
                });
                assert_expected_encode_limit(small_result.unwrap_err(), resource, $label, "writer");
                assert_expected_encode_limit(large_result.unwrap_err(), resource, $label, "writer");
                assert_eq!(small_writer, b"prefix", "{} writer changed prefix", $label);
                assert_eq!(large_writer, b"prefix", "{} writer changed prefix", $label);
                assert_bounded_direct_helper_allocations(
                    $label,
                    "writer",
                    resource,
                    small_bytes,
                    large_bytes,
                );
            }
        }};
    }

    check_helpers!(
        "ValueWirePayloadRefV1 scalar",
        &small_payload_ref,
        &large_payload_ref
    );
    check_helpers!("ValueWireRefV1 scalar", &small_wire_ref, &large_wire_ref);
    check_helpers!("ValueWirePayloadV1 scalar", &small_payload, &large_payload);
    check_helpers!("ValueWireV1 scalar", &small_wire, &large_wire);
    check_helpers!(
        "ValueWirePayloadRefV1 collection",
        &small_collection_payload_ref,
        &large_collection_payload_ref
    );
    check_helpers!(
        "ValueWireRefV1 collection",
        &small_collection_wire_ref,
        &large_collection_wire_ref
    );
    check_helpers!(
        "ValueWirePayloadV1 collection",
        &small_collection_payload,
        &large_collection_payload
    );
    check_helpers!(
        "ValueWireV1 collection",
        &small_collection_wire,
        &large_collection_wire
    );
}

#[test]
fn test_json_number_payload_construction_avoids_per_float_string_allocation() {
    let preserves_number_lexeme = from_str::<Number>("1.2500").expect("number must parse")
        != from_str::<Number>("1.25").expect("number must parse");
    let mut allocation_bytes = Vec::new();
    for floating in [false, true] {
        let label = if floating { "float" } else { "integer" };
        for size in [1, 256, 4096] {
            let value = json_number_fixture(size, floating);
            let bytes = measured_json_payload_construction_bytes(&value);
            eprintln!("json_number_allocation: {label} size={size} bytes={bytes}");
            allocation_bytes.push((floating, size, bytes));
        }
    }
    let allocation = |floating, size| {
        allocation_bytes
            .iter()
            .find_map(|(matches_float, matches_size, bytes)| {
                (*matches_float == floating && *matches_size == size).then_some(*bytes)
            })
            .expect("allocation sample must exist")
    };
    if preserves_number_lexeme {
        // serde_json's arbitrary_precision representation allocates while
        // validating each number. Preserve correctness and serialization
        // coverage for that opt-in feature without conflating it with the
        // standard Number formatter's allocation regression assertion.
        for size in [1, 256, 4096] {
            let value = json_number_fixture(size, true);
            let payload = ValueWirePayloadRefV1::from_value(&value)
                .expect("arbitrary-precision number fixture must be accepted");
            assert!(payload.to_json_vec().is_ok(), "number fixture must encode");
        }
    } else {
        assert!(
            allocation(true, 4096) <= allocation(true, 256).saturating_add(4096),
            "float validation allocation grew with element count: 256={} bytes, 4096={} bytes",
            allocation(true, 256),
            allocation(true, 4096),
        );
    }
    assert!(
        allocation(false, 4096) <= allocation(false, 256).saturating_add(4096),
        "integer control allocation grew with element count: 256={} bytes, 4096={} bytes",
        allocation(false, 256),
        allocation(false, 4096),
    );
}

#[test]
fn test_named_helpers_reject_large_decimal_before_formatting() {
    let small_digits = "9".repeat(4096);
    let large_digits = "9".repeat(128 * 1024);
    let small_coefficient =
        BigInt::parse_bytes(small_digits.as_bytes(), 10).expect("small coefficient must parse");
    let large_coefficient =
        BigInt::parse_bytes(large_digits.as_bytes(), 10).expect("large coefficient must parse");
    let small_scalar = NamedValue::new(
        "decimal",
        Value::BigDecimal(BigDecimal::new(small_coefficient.clone(), 2)),
    );
    let large_scalar = NamedValue::new(
        "decimal",
        Value::BigDecimal(BigDecimal::new(large_coefficient.clone(), 2)),
    );
    let small_collection = NamedMultiValues::new(
        "decimals",
        MultiValues::BigDecimal(vec![BigDecimal::new(small_coefficient, 2)]),
    );
    let large_collection = NamedMultiValues::new(
        "decimals",
        MultiValues::BigDecimal(vec![BigDecimal::new(large_coefficient, 2)]),
    );

    macro_rules! check_named_helpers {
        ($label:literal, $small:expr, $large:expr) => {{
            let profiles = [
                (
                    JsonEncodeLimits::builder().max_string_bytes(64).build(),
                    JsonResource::StringBytes,
                ),
                (
                    JsonEncodeLimits::builder().max_payload_bytes(128).build(),
                    JsonResource::PayloadBytes,
                ),
                (
                    JsonEncodeLimits::builder().max_output_bytes(256).build(),
                    JsonResource::OutputBytes,
                ),
            ];
            for (limits, resource) in profiles {
                let (small_result, small_bytes) =
                    measured_allocations(|| $small.to_json_vec_with_limits(limits));
                let (large_result, large_bytes) =
                    measured_allocations(|| $large.to_json_vec_with_limits(limits));
                assert_expected_encode_limit(small_result.unwrap_err(), resource, $label, "vector");
                assert_expected_encode_limit(large_result.unwrap_err(), resource, $label, "vector");
                assert_bounded_direct_helper_allocations(
                    $label,
                    "vector",
                    resource,
                    small_bytes,
                    large_bytes,
                );

                let mut small_writer = b"prefix".to_vec();
                let mut large_writer = b"prefix".to_vec();
                let (small_result, small_bytes) = measured_allocations(|| {
                    $small.to_json_writer_with_limits(&mut small_writer, limits)
                });
                let (large_result, large_bytes) = measured_allocations(|| {
                    $large.to_json_writer_with_limits(&mut large_writer, limits)
                });
                assert_expected_encode_limit(small_result.unwrap_err(), resource, $label, "writer");
                assert_expected_encode_limit(large_result.unwrap_err(), resource, $label, "writer");
                assert_eq!(small_writer, b"prefix", "{} writer changed prefix", $label);
                assert_eq!(large_writer, b"prefix", "{} writer changed prefix", $label);
                assert_bounded_direct_helper_allocations(
                    $label,
                    "writer",
                    resource,
                    small_bytes,
                    large_bytes,
                );
            }

            let small_default = $small
                .to_json_vec()
                .expect("small named fixture fits the default profile");
            assert_eq!(
                small_default,
                $small
                    .to_json_vec_with_limits(JsonEncodeLimits::new())
                    .expect("small named fixture fits an explicit wide profile"),
                "default named vector must match the explicit wide profile",
            );
            let mut default_writer = Vec::new();
            $small
                .to_json_writer(&mut default_writer)
                .expect("default named writer helper succeeds");
            assert_eq!(
                default_writer, small_default,
                "default named writer must match vector output"
            );
        }};
    }

    check_named_helpers!("NamedValue", &small_scalar, &large_scalar);
    check_named_helpers!("NamedMultiValues", &small_collection, &large_collection);
}

#[test]
fn test_named_helpers_keep_name_and_complete_output_limits() {
    let named = NamedValue::new("toolong", Value::Int32(7));
    let limits = JsonEncodeLimits::builder().max_string_bytes(3).build();
    assert_expected_encode_limit(
        named.to_json_vec_with_limits(limits).unwrap_err(),
        JsonResource::StringBytes,
        "NamedValue name",
        "vector",
    );
    let mut writer = b"prefix".to_vec();
    assert_expected_encode_limit(
        named
            .to_json_writer_with_limits(&mut writer, limits)
            .unwrap_err(),
        JsonResource::StringBytes,
        "NamedValue name",
        "writer",
    );
    assert_eq!(
        writer, b"prefix",
        "name rejection must leave writer prefix unchanged"
    );

    let small = NamedValue::new("ok", Value::Int32(7));
    let wide = JsonEncodeLimits::new();
    let encoded = small
        .to_json_vec_with_limits(wide)
        .expect("wide profile must encode the full wrapper");
    let complete_len = encoded.len();
    assert!(
        small
            .to_json_vec_with_limits(
                JsonEncodeLimits::builder()
                    .max_output_bytes(complete_len)
                    .build()
            )
            .is_ok(),
        "the exact complete output limit must accept the named wrapper",
    );
    assert!(
        matches!(
            small
                .to_json_vec_with_limits(
                    JsonEncodeLimits::builder()
                        .max_output_bytes(complete_len - 1)
                        .build(),
                )
                .unwrap_err(),
            ValueWireEncodeError::Budget(BudgetError::Insufficient {
                resource: JsonResource::OutputBytes,
                ..
            })
        ),
        "one byte below the complete output size must report insufficient output budget",
    );
}
