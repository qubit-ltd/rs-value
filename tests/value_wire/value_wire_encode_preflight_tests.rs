// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Contract tests for conservative Wire V1 encoding preflight checks.

use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::process::Command;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::Utc;
use num_bigint::BigInt;
use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::Observation;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonEncodeSession;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;
use qubit_datatype::DataType;
use qubit_json::encode::JsonEncoder;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWireV1;
use url::Url;

#[test]
fn test_preflight_key_payload_boundary() {
    let value = Value::StringMap(HashMap::from([("abc".to_owned(), String::new())]));
    let limits = JsonEncodeLimits::builder().max_payload_bytes(2_usize).build();
    assert!(ValueWireEncodePreflight::new(limits).check_value(&value).is_err());
    let limits = JsonEncodeLimits::builder()
        .max_payload_bytes(3_usize)
        .max_nodes(2_usize)
        .build();
    assert!(ValueWireEncodePreflight::new(limits).check_value(&value).is_ok());
}

#[test]
fn test_preflight_key_output_boundaries_and_utf8_lower_bound() {
    let value = Value::StringMap(HashMap::from([("abc".to_owned(), String::new())]));
    for (maximum, accepted) in [(9, false), (10, true)] {
        let limits = JsonEncodeLimits::builder()
            .max_output_bytes(maximum)
            .max_nodes(2)
            .build();
        assert_eq!(
            ValueWireEncodePreflight::new(limits).check_value(&value).is_ok(),
            accepted
        );
    }

    let json = Value::Json(serde_json::json!({"abc": null}));
    for (maximum, accepted) in [(8, false), (9, true)] {
        let limits = JsonEncodeLimits::builder().max_output_bytes(maximum).build();
        assert_eq!(
            ValueWireEncodePreflight::new(limits).check_value(&json).is_ok(),
            accepted
        );
    }

    for key in ["", "键", "quote\"slash\\"] {
        let value = Value::StringMap(HashMap::from([(key.to_owned(), String::new())]));
        let wire = ValueWireV1::try_from(value.clone()).expect("map must have a wire representation");
        let encoded = wire
            .to_json_vec_with_limits(JsonEncodeLimits::new())
            .expect("encoding must succeed");
        let limits = JsonEncodeLimits::builder().max_output_bytes(encoded.len()).build();
        ValueWireEncodePreflight::new(limits)
            .check_value(&value)
            .expect("key UTF-8 and quote bytes form a safe output lower bound");
    }

    let empty = Value::StringMap(HashMap::new());
    ValueWireEncodePreflight::new(JsonEncodeLimits::builder().max_payload_bytes(0).build())
        .check_value(&empty)
        .expect("an empty object has no key payload");
}

#[test]
fn test_preflight_key_payload_accumulates_and_failed_calls_roll_back() {
    let limits = JsonEncodeLimits::builder().max_payload_bytes(6).build();
    let map = Value::StringMap(HashMap::from([("abc".to_owned(), String::new())]));
    assert_three_key_calls(
        ValueWireEncodePreflight::new(limits),
        |checker| checker.check_value(&map),
        "failed scalar check must roll back key bytes",
    );
    assert_three_key_calls(
        ValueWireEncodePreflight::new(limits),
        |checker| {
            checker.check_values(&MultiValues::StringMap(vec![HashMap::from([(
                "abc".to_owned(),
                String::new(),
            )])]))
        },
        "failed collection check must roll back key bytes",
    );
    assert_three_key_calls(
        ValueWireEncodePreflight::new(limits),
        |checker| checker.check_container(&ValueContainer::Scalar(map.clone())),
        "failed container check must roll back key bytes",
    );
}

fn assert_three_key_calls<F>(mut checker: ValueWireEncodePreflight, mut check: F, rollback_message: &str)
where
    F: FnMut(&mut ValueWireEncodePreflight) -> Result<(), MeasuredBudgetError<JsonResource, usize>>,
{
    check(&mut checker).expect("first key-bearing value fits");
    check(&mut checker).expect("second key-bearing value exactly fits");
    assert_limit_exceeded(
        check(&mut checker).expect_err("third key must exceed cumulative payload"),
        JsonResource::PayloadBytes,
    );
    checker
        .check_value(&Value::String(String::new()))
        .expect(rollback_message);
}

#[test]
fn test_preflight_fixed_object_key_payload_and_output_boundaries() {
    let duration = Value::Duration(Duration::ZERO);
    for (maximum, accepted) in [(10, false), (11, true)] {
        let limits = JsonEncodeLimits::builder()
            .max_payload_bytes(maximum)
            .max_nodes(3)
            .build();
        assert_eq!(
            ValueWireEncodePreflight::new(limits).check_value(&duration).is_ok(),
            accepted
        );
    }
    for (maximum, accepted) in [(19, false), (20, true)] {
        let limits = JsonEncodeLimits::builder()
            .max_output_bytes(maximum)
            .max_nodes(3)
            .build();
        assert_eq!(
            ValueWireEncodePreflight::new(limits).check_value(&duration).is_ok(),
            accepted
        );
    }

    let decimal = Value::BigDecimal(BigDecimal::new(BigInt::from(0), 0));
    for (maximum, accepted) in [(17, false), (18, true)] {
        let limits = JsonEncodeLimits::builder()
            .max_payload_bytes(maximum)
            .max_nodes(3)
            .build();
        assert_eq!(
            ValueWireEncodePreflight::new(limits).check_value(&decimal).is_ok(),
            accepted
        );
    }
    for (maximum, accepted) in [(28, false), (29, true)] {
        let limits = JsonEncodeLimits::builder()
            .max_output_bytes(maximum)
            .max_nodes(3)
            .build();
        assert_eq!(
            ValueWireEncodePreflight::new(limits).check_value(&decimal).is_ok(),
            accepted
        );
    }
}

#[cfg(feature = "json")]
const DEEP_PREFLIGHT_CHILD_ENV: &str = "QUBIT_VALUE_DEEP_PREFLIGHT_CHILD";

#[test]
fn test_preflight_accepts_encoded_uint128_with_small_number_limit() {
    let value = Value::UInt128(123);
    let limits = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_number_bytes(2)
        .build();
    let wire = ValueWireV1::try_from(value.clone()).expect("the unsigned integer must have a V1 wire value");
    assert!(wire.to_json_vec_with_limits(limits).is_ok());
    assert!(ValueWireEncodePreflight::new(limits).check_value(&value).is_ok());
}

#[test]
fn test_preflight_accepts_scientific_float_with_small_number_limit() {
    let value = Value::Float64(1e100);
    let limits = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_number_bytes(10)
        .build();
    let wire = ValueWireV1::try_from(value.clone()).expect("the finite float must have a V1 wire value");
    assert!(wire.to_json_vec_with_limits(limits).is_ok());
    assert!(ValueWireEncodePreflight::new(limits).check_value(&value).is_ok());
}

#[test]
fn test_preflight_counts_duration_payload_nodes_and_keys() {
    let value = Value::Duration(Duration::new(12, 345));
    let limits = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_nodes(3)
        .max_key_bytes(5)
        .build();
    ValueWireEncodePreflight::new(limits)
        .check_value(&value)
        .expect("duration object and both numeric fields fit");
    let too_small = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_nodes(2)
        .build();
    assert!(ValueWireEncodePreflight::new(too_small).check_value(&value).is_err());
}

#[test]
fn test_preflight_counts_big_decimal_payload_nodes() {
    let value = Value::BigDecimal("12.5".parse::<BigDecimal>().expect("the decimal literal must parse"));
    let limits = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_nodes(3)
        .max_key_bytes(11)
        .build();
    ValueWireEncodePreflight::new(limits)
        .check_value(&value)
        .expect("decimal object and both fields fit");
    let too_small = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_nodes(2)
        .build();
    assert!(ValueWireEncodePreflight::new(too_small).check_value(&value).is_err());
}

#[test]
fn test_preflight_big_decimal_coefficient_respects_string_payload_and_output_limits() {
    let coefficient = BigInt::from(10_u8).pow(4095);
    let value = Value::BigDecimal(BigDecimal::new(coefficient, 2));
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
        let error = ValueWireEncodePreflight::new(limits)
            .check_value(&value)
            .expect_err("the large coefficient must be rejected before encoding");
        assert_limit_exceeded(error, resource);
    }
}

#[test]
fn test_preflight_big_decimal_small_coefficients_and_rollback() {
    let limits = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_string_bytes(64)
        .max_number_bytes(1)
        .build();
    for coefficient in [0_i64, 7, -7] {
        let value = Value::BigDecimal(BigDecimal::new(BigInt::from(coefficient), 2));
        let wire = ValueWireV1::try_from(value.clone()).expect("small decimal should have a wire value");
        wire.to_json_vec_with_limits(limits)
            .expect("small decimal coefficient is encoded as a string, independent of number limit");
        ValueWireEncodePreflight::new(limits)
            .check_value(&value)
            .expect("small coefficient lower bound must fit");
    }

    let limits = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_string_bytes(64)
        .build();
    let small = Value::BigDecimal(BigDecimal::new(BigInt::from(1), 2));
    let large = Value::BigDecimal(BigDecimal::new(BigInt::from(10_u8).pow(4095), 2));
    let mut checker = ValueWireEncodePreflight::new(limits);
    checker.check_value(&small).expect("initial small coefficient must fit");
    assert_limit_exceeded(
        checker
            .check_value(&large)
            .expect_err("large coefficient must be rejected"),
        JsonResource::StringBytes,
    );
    checker
        .check_value(&small)
        .expect("failed check must restore all counters");

    let mut reference = ValueWireEncodePreflight::new(limits);
    reference.check_value(&small).expect("reference first value must fit");
    reference.check_value(&small).expect("reference second value must fit");
}

#[test]
fn test_preflight_encoding_success_implies_acceptance_at_output_boundaries() {
    let values = [
        Value::Int128(i128::MIN),
        Value::UInt128(u128::MAX),
        Value::Float32(f32::from_bits(1)),
        Value::Float32(-0.0),
        Value::Float32(f32::INFINITY),
        Value::Float64(f64::from_bits(1)),
        Value::Float64(-0.0),
        Value::Float64(f64::NAN),
        Value::Float64(f64::INFINITY),
        Value::Duration(Duration::new(u64::MAX, 999_999_999)),
        Value::Unset(DataType::UInt128),
        Value::BigInteger(BigInt::from_bytes_le(num_bigint::Sign::Minus, &[0xff; 256])),
        Value::BigDecimal(BigDecimal::new(BigInt::from(0), 2)),
        Value::BigDecimal(BigDecimal::new(BigInt::from(-7), 150_000)),
        Value::BigDecimal(BigDecimal::new(BigInt::from(10_u8).pow(255), -150_001)),
        Value::Json(serde_json::json!([null, true, 123, "mixed", {"nested": [false]}])),
    ];

    let positive = Value::Int32(7);
    let positive_wire = ValueWireV1::try_from(positive.clone()).expect("int32 must have a V1 wire value");
    let encoded = JsonEncoder::new(JsonEncodeSession::from_limits(
        JsonEncodeLimits::<JsonResource, usize>::new(),
    ))
    .to_vec(&positive_wire)
    .expect("the fixed positive sample must encode successfully");
    ValueWireEncodePreflight::new(JsonEncodeLimits::new())
        .check_value(&positive)
        .expect("the fixed positive sample must pass preflight");
    assert!(!encoded.is_empty());

    for value in values {
        let wire = match ValueWireV1::try_from(value.clone()) {
            Ok(wire) => wire,
            Err(_) => continue,
        };
        let unbounded: JsonEncodeLimits = JsonEncodeLimits::new();
        let encoded = match JsonEncoder::new(JsonEncodeSession::from_limits(unbounded)).to_vec(&wire) {
            Ok(encoded) => encoded,
            Err(_) => continue,
        };
        let size = encoded.len();
        for max_output_bytes in [size.saturating_sub(1), size, size.saturating_add(1)] {
            let limits = JsonEncodeLimits::builder()
                .max_output_bytes(max_output_bytes)
                .max_depth(64)
                .max_nodes(4096)
                .max_sequence_items(32)
                .max_map_entries(32)
                .max_key_bytes(4096)
                .max_string_bytes(4096)
                .max_number_bytes(4096)
                .max_payload_bytes(4 * 1024 * 1024)
                .build();
            if JsonEncoder::new(JsonEncodeSession::from_limits(limits))
                .to_vec(&wire)
                .is_ok()
            {
                ValueWireEncodePreflight::new(limits)
                    .check_value(&value)
                    .expect("successful encoding must imply preflight acceptance");
            }
        }
    }
}

#[test]
fn test_preflight_encoding_success_implies_acceptance_for_collections_and_limits() {
    let values = [
        ValueContainer::Collection(MultiValues::Int128(Vec::new())),
        ValueContainer::Collection(MultiValues::String(vec![String::new()])),
        ValueContainer::Collection(MultiValues::Json(vec![
            serde_json::Value::Null,
            serde_json::json!({"items": [true, 1, "text"]}),
        ])),
    ];
    let dimension_profiles = [
        JsonEncodeLimits::builder().max_depth(0).build(),
        JsonEncodeLimits::builder().max_depth(1).build(),
        JsonEncodeLimits::builder().max_nodes(0).build(),
        JsonEncodeLimits::builder().max_nodes(1).build(),
        JsonEncodeLimits::builder().max_sequence_items(0).build(),
        JsonEncodeLimits::builder().max_sequence_items(1).build(),
        JsonEncodeLimits::builder().max_map_entries(0).build(),
        JsonEncodeLimits::builder().max_map_entries(1).build(),
        JsonEncodeLimits::builder().max_key_bytes(0).build(),
        JsonEncodeLimits::builder().max_key_bytes(1).build(),
        JsonEncodeLimits::builder().max_string_bytes(0).build(),
        JsonEncodeLimits::builder().max_string_bytes(1).build(),
        JsonEncodeLimits::builder().max_number_bytes(0).build(),
        JsonEncodeLimits::builder().max_number_bytes(1).build(),
        JsonEncodeLimits::builder().max_payload_bytes(0).build(),
        JsonEncodeLimits::builder().max_payload_bytes(1).build(),
        JsonEncodeLimits::builder().max_output_bytes(0).build(),
        JsonEncodeLimits::builder().max_output_bytes(1).build(),
        JsonEncodeLimits::builder()
            .max_depth(64)
            .max_nodes(4096)
            .max_sequence_items(32)
            .max_map_entries(32)
            .max_key_bytes(4096)
            .max_string_bytes(4096)
            .max_number_bytes(4096)
            .max_payload_bytes(4 * 1024 * 1024)
            .max_output_bytes(4 * 1024 * 1024)
            .build(),
    ];

    let positive = ValueContainer::Collection(MultiValues::Int32(vec![1, 2]));
    let wire = ValueWireV1::try_from(positive.clone()).expect("int32 collection must have a V1 wire value");
    JsonEncoder::new(JsonEncodeSession::from_limits(
        JsonEncodeLimits::<JsonResource, usize>::new(),
    ))
    .to_vec(&wire)
    .expect("the fixed collection positive sample must encode successfully");
    ValueWireEncodePreflight::new(JsonEncodeLimits::new())
        .check_container(&positive)
        .expect("the fixed collection positive sample must pass preflight");

    for value in values {
        for limits in &dimension_profiles {
            let wire = match ValueWireV1::try_from(value.clone()) {
                Ok(wire) => wire,
                Err(_) => continue,
            };
            if JsonEncoder::new(JsonEncodeSession::from_limits(*limits))
                .to_vec(&wire)
                .is_ok()
            {
                ValueWireEncodePreflight::new(*limits)
                    .check_container(&value)
                    .expect("successful encoding must imply preflight acceptance");
            }
        }
    }
}

#[test]
fn test_preflight_counts_unset_wire_tag_as_string() {
    let value = Value::Unset(DataType::UInt128);
    let limits = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_string_bytes("uint128".len())
        .build();
    ValueWireEncodePreflight::new(limits)
        .check_value(&value)
        .expect("the generated V1 unset tag fits its string budget");
}

/// Verifies that an error identifies the expected exhausted JSON resource.
fn assert_limit_exceeded(error: MeasuredBudgetError<JsonResource, usize>, expected_resource: JsonResource) {
    assert!(
        matches!(
            error,
            MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
                resource,
                ..
            }) if resource == expected_resource
        ),
        "expected a {expected_resource:?} limit error, got {error:?}",
    );
}

/// Verifies a conservative cumulative failure without discarding its measured
/// lower bound.
fn assert_lower_bound_limit(
    error: MeasuredBudgetError<JsonResource, usize>,
    expected_resource: JsonResource,
    expected_observed: usize,
    expected_maximum: usize,
) {
    assert!(
        matches!(
            error,
            MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
                resource,
                observed: Observation::AtLeast(observed),
                maximum,
            }) if resource == expected_resource
                && observed == expected_observed
                && maximum == expected_maximum
        ),
        "expected {expected_resource:?} at least {expected_observed}/{expected_maximum}, got {error:?}",
    );
}

/// Verifies a point-limit failure retains the exact native measurement.
fn assert_exact_limit(
    error: MeasuredBudgetError<JsonResource, usize>,
    expected_resource: JsonResource,
    expected_observed: usize,
    expected_maximum: usize,
) {
    assert!(
        matches!(
            error,
            MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
                resource,
                observed: Observation::Exact(observed),
                maximum,
            }) if resource == expected_resource
                && observed == expected_observed
                && maximum == expected_maximum
        ),
        "expected {expected_resource:?} exactly {expected_observed}/{expected_maximum}, got {error:?}",
    );
}

#[test]
fn test_new_u64_limits_saturates_unrepresentable_limits() {
    let oversized_limit = (usize::MAX as u64).saturating_add(1);
    let limits = JsonEncodeLimits::<JsonResource, u64>::builder()
        .max_output_bytes(oversized_limit)
        .max_depth(oversized_limit)
        .max_nodes(oversized_limit)
        .max_sequence_items(oversized_limit)
        .max_map_entries(oversized_limit)
        .max_key_bytes(oversized_limit)
        .max_string_bytes(oversized_limit)
        .max_number_bytes(oversized_limit)
        .max_payload_bytes(oversized_limit)
        .build();
    let value = Value::Json(serde_json::json!({
        "items": [1, "portable"]
    }));
    let mut checker = ValueWireEncodePreflight::new_u64_limits(limits);

    checker
        .check_value(&value)
        .expect("limits above the native range must saturate instead of wrapping");
}

#[test]
fn test_new_value_limits_enforces_value_budget_without_outer_output_limit() {
    let limits = JsonValueLimits::builder().max_nodes(1_usize).build();
    let mut checker = ValueWireEncodePreflight::new_value_limits(limits);

    checker
        .check_value(&Value::String("an arbitrarily long value".to_owned()))
        .expect("value-only limits must not introduce an outer output limit");
    let error = checker
        .check_value(&Value::Bool(true))
        .expect_err("the second value must exceed the cumulative node limit");

    assert_lower_bound_limit(error, JsonResource::Nodes, 2, 1);
}

#[test]
fn test_new_u64_limits_preserves_unconfigured_dimensions() {
    let limits = JsonEncodeLimits::<JsonResource, u64>::new();
    let mut checker = ValueWireEncodePreflight::new_u64_limits(limits);
    let value = Value::Json(serde_json::json!({
        "items": [null, true, 42, "ready"]
    }));

    checker
        .check_value(&value)
        .expect("an unconfigured u64 profile must remain unbounded");
}

#[test]
fn test_check_value_traverses_every_scalar_shape_and_accumulates_nodes() {
    let date = NaiveDate::from_ymd_opt(2026, 9, 7).expect("valid date");
    let time = NaiveTime::from_hms_opt(12, 34, 56).expect("valid time");
    let datetime = date.and_time(time);
    let instant = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc);
    let values = vec![
        (Value::Unset(DataType::Bool), 1_usize),
        (Value::Bool(true), 1),
        (Value::Char('λ'), 1),
        (Value::String("ready".to_owned()), 1),
        (Value::Int8(i8::MIN), 1),
        (Value::Int16(i16::MIN), 1),
        (Value::Int32(i32::MIN), 1),
        (Value::Int64(i64::MIN), 1),
        (Value::Int128(i128::MIN), 1),
        (Value::UInt8(u8::MAX), 1),
        (Value::UInt16(u16::MAX), 1),
        (Value::UInt32(u32::MAX), 1),
        (Value::UInt64(u64::MAX), 1),
        (Value::UInt128(u128::MAX), 1),
        (Value::Float32(1.5), 1),
        (Value::Float64(-2.25), 1),
        (Value::BigInteger(BigInt::from(0)), 1),
        (Value::BigInteger(BigInt::from(-7)), 1),
        (Value::BigDecimal("7.5".parse::<BigDecimal>().expect("decimal")), 3),
        (Value::Duration(Duration::from_secs(1)), 3),
        (Value::Date(date), 1),
        (Value::Time(time), 1),
        (Value::DateTime(datetime), 1),
        (Value::Instant(instant), 1),
        (Value::Url(Url::parse("https://example.com/path").expect("URL")), 1),
        (
            Value::StringMap(HashMap::from([("key".to_owned(), "value".to_owned())])),
            2,
        ),
        (
            Value::Json(serde_json::json!({
                "items": [null, true, 42, "ready"]
            })),
            6,
        ),
    ];
    let maximum_nodes = values.iter().map(|(_, nodes)| nodes).sum::<usize>();
    let mut checker = ValueWireEncodePreflight::new(JsonEncodeLimits::builder().max_nodes(maximum_nodes).build());

    for (value, _) in &values {
        checker
            .check_value(value)
            .expect("each supported scalar shape must fit its exact node budget");
    }
    let error = checker
        .check_value(&Value::Bool(false))
        .expect_err("one more scalar must exceed the accumulated node budget");

    assert_lower_bound_limit(error, JsonResource::Nodes, maximum_nodes + 1, maximum_nodes);
}

#[test]
fn test_check_values_traverses_specialized_collection_shapes() {
    let collections = vec![
        (MultiValues::Bool(vec![true, false]), 3_usize),
        (MultiValues::Char(vec!['a', 'λ']), 3),
        (MultiValues::String(vec!["a".to_owned(), "bc".to_owned()]), 3),
        (
            MultiValues::StringMap(vec![HashMap::from([("key".to_owned(), "value".to_owned())])]),
            3,
        ),
        (
            MultiValues::Json(vec![serde_json::json!({
                "items": [null, true, 42, "ready"]
            })]),
            7,
        ),
        (MultiValues::UInt128(vec![1, 2]), 3),
    ];
    let maximum_nodes = collections.iter().map(|(_, nodes)| nodes).sum::<usize>();
    let mut checker = ValueWireEncodePreflight::new(JsonEncodeLimits::builder().max_nodes(maximum_nodes).build());

    for (values, _) in &collections {
        checker
            .check_values(values)
            .expect("each specialized collection shape must fit its node budget");
    }
    let error = checker
        .check_value(&Value::Bool(false))
        .expect_err("one more scalar must exceed the accumulated node budget");

    assert_lower_bound_limit(error, JsonResource::Nodes, maximum_nodes + 1, maximum_nodes);
}

#[test]
fn test_check_value_reports_each_point_limit_precisely() {
    let cases = [
        (
            Value::String("abc".to_owned()),
            JsonEncodeLimits::builder().max_depth(0_usize).build(),
            JsonResource::Depth,
            1,
            0,
        ),
        (
            Value::String("abc".to_owned()),
            JsonEncodeLimits::builder().max_string_bytes(2_usize).build(),
            JsonResource::StringBytes,
            3,
            2,
        ),
        (
            Value::UInt64(123),
            JsonEncodeLimits::builder().max_number_bytes(2_usize).build(),
            JsonResource::NumberBytes,
            3,
            2,
        ),
        (
            Value::StringMap(HashMap::from([("long".to_owned(), "value".to_owned())])),
            JsonEncodeLimits::builder().max_key_bytes(3_usize).build(),
            JsonResource::KeyBytes,
            4,
            3,
        ),
        (
            Value::StringMap(HashMap::from([
                ("a".to_owned(), "1".to_owned()),
                ("b".to_owned(), "2".to_owned()),
            ])),
            JsonEncodeLimits::builder().max_map_entries(1_usize).build(),
            JsonResource::MapEntries,
            2,
            1,
        ),
    ];

    for (value, limits, resource, observed, maximum) in cases {
        let mut checker = ValueWireEncodePreflight::new(limits);
        let error = checker
            .check_value(&value)
            .expect_err("the configured point limit must reject the value");
        assert_exact_limit(error, resource, observed, maximum);
    }

    let mut checker = ValueWireEncodePreflight::new(JsonEncodeLimits::builder().max_sequence_items(1_usize).build());
    let error = checker
        .check_values(&MultiValues::Bool(vec![true, false]))
        .expect_err("the sequence-item limit must reject two values");
    assert_exact_limit(error, JsonResource::SequenceItems, 2, 1);
}

#[test]
fn test_check_value_reports_each_cumulative_limit_precisely() {
    let cases = [
        (
            JsonEncodeLimits::builder().max_nodes(0_usize).build(),
            JsonResource::Nodes,
            1,
        ),
        (
            JsonEncodeLimits::builder().max_payload_bytes(2_usize).build(),
            JsonResource::PayloadBytes,
            3,
        ),
        (
            JsonEncodeLimits::builder().max_output_bytes(4_usize).build(),
            JsonResource::OutputBytes,
            5,
        ),
    ];

    for (limits, resource, observed) in cases {
        let mut checker = ValueWireEncodePreflight::new(limits);
        let error = checker
            .check_value(&Value::String("abc".to_owned()))
            .expect_err("the configured cumulative limit must reject the value");
        assert_lower_bound_limit(error, resource, observed, observed - 1);
    }
}

#[test]
fn test_preflight_accepts_deep_arrays_with_tight_payload() {
    let mut payload = serde_json::Value::Bool(true);
    for _ in 0..32 {
        payload = serde_json::Value::Array(vec![payload]);
    }
    let value = Value::Json(payload);
    let limits = JsonEncodeLimits::builder().max_payload_bytes(32).build();
    let wire = ValueWireV1::try_from(value.clone()).expect("JSON value should have a Wire V1 representation");

    wire.to_json_vec_with_limits(limits)
        .expect("the encoded payload fits the configured limit");
    ValueWireEncodePreflight::new(limits)
        .check_value(&value)
        .expect("array punctuation must not consume the payload budget");
}

#[test]
fn test_preflight_accepts_deep_arrays_at_exact_output_size() {
    let mut payload = serde_json::Value::Bool(true);
    for _ in 0..60 {
        payload = serde_json::Value::Array(vec![payload]);
    }
    let value = Value::Json(payload);
    let wire = ValueWireV1::try_from(value.clone()).expect("JSON value should have a Wire V1 representation");
    let encoded = wire
        .to_json_vec_with_limits(JsonEncodeLimits::new())
        .expect("unlimited encoding should succeed");
    let limits = JsonEncodeLimits::builder().max_output_bytes(encoded.len()).build();

    wire.to_json_vec_with_limits(limits)
        .expect("encoding succeeds at its exact output length");
    ValueWireEncodePreflight::new(limits)
        .check_value(&value)
        .expect("preflight output lower bound must not exceed encoded output");
}

#[test]
fn test_empty_strings_do_not_consume_payload_budget() {
    let limits = JsonEncodeLimits::builder().max_payload_bytes(0_usize).build();
    let mut checker = ValueWireEncodePreflight::new(limits);

    checker
        .check_value(&Value::String(String::new()))
        .expect("an empty string has no payload text bytes");
    checker
        .check_values(&MultiValues::String(vec![String::new()]))
        .expect("an empty string in a collection has no payload text bytes");
}

#[test]
fn test_failed_string_check_rolls_back_payload_before_next_check() {
    let limits = JsonEncodeLimits::builder().max_payload_bytes(3_usize).build();
    let mut checker = ValueWireEncodePreflight::new(limits);

    checker
        .check_value(&Value::String("ab".to_owned()))
        .expect("the first string fits");
    let error = checker
        .check_value(&Value::String("cd".to_owned()))
        .expect_err("the second string exceeds the cumulative payload limit");
    assert_limit_exceeded(error, JsonResource::PayloadBytes);
    checker
        .check_value(&Value::String("x".to_owned()))
        .expect("failed checks must not consume payload budget");
}

#[test]
fn test_preflight_lower_bounds_for_empty_json_containers_and_map_text() {
    let cases = [
        Value::Json(serde_json::Value::Array(Vec::new())),
        Value::Json(serde_json::Value::Object(serde_json::Map::new())),
        Value::Json(serde_json::json!({ "": "" })),
        Value::Json(serde_json::json!({ "text": "long value" })),
        Value::Json(serde_json::json!(12345678901234567890_u64)),
        Value::UInt128(u128::MAX),
        Value::StringMap(HashMap::from([(String::new(), String::new())])),
    ];

    for value in cases {
        let wire = ValueWireV1::try_from(value.clone()).expect("value should have a Wire V1 representation");
        let encoded = wire
            .to_json_vec_with_limits(JsonEncodeLimits::new())
            .expect("unlimited encoding should succeed");
        let limits = JsonEncodeLimits::builder().max_output_bytes(encoded.len()).build();
        wire.to_json_vec_with_limits(limits)
            .expect("encoding succeeds at its exact output length");
        ValueWireEncodePreflight::new(limits)
            .check_value(&value)
            .expect("preflight lower bound must fit when bounded encoding succeeds");
    }
}

#[test]
fn test_check_value_accumulates_successful_calls() {
    let limits = JsonEncodeLimits::builder().max_nodes(2_usize).build();
    let mut checker = ValueWireEncodePreflight::new(limits);

    checker
        .check_value(&Value::Bool(true))
        .expect("the first node must fit");
    checker
        .check_value(&Value::Bool(false))
        .expect("the second cumulative node must fit");
    let error = checker
        .check_value(&Value::Bool(true))
        .expect_err("the third cumulative node must exceed the limit");

    assert_limit_exceeded(error, JsonResource::Nodes);
}

#[test]
fn test_check_value_failure_does_not_mutate_accumulated_state() {
    let limits = JsonEncodeLimits::builder().max_nodes(3_usize).build();
    let mut checker = ValueWireEncodePreflight::new(limits);
    let nested = Value::StringMap(HashMap::from([
        ("first".to_owned(), "one".to_owned()),
        ("second".to_owned(), "two".to_owned()),
    ]));

    checker
        .check_value(&Value::Bool(true))
        .expect("the initial node must fit");
    let error = checker
        .check_value(&nested)
        .expect_err("the nested value must exceed the node limit");
    assert_limit_exceeded(error, JsonResource::Nodes);

    checker
        .check_value(&Value::Bool(false))
        .expect("a failed value check must restore the previous node count");
}

#[test]
fn test_check_values_failure_does_not_mutate_accumulated_state() {
    let limits = JsonEncodeLimits::builder().max_payload_bytes(1_usize).build();
    let mut checker = ValueWireEncodePreflight::new(limits);

    checker
        .check_value(&Value::Bool(true))
        .expect("the initial zero-byte boolean payload must fit");
    let error = checker
        .check_values(&MultiValues::Int32(vec![1, 2]))
        .expect_err("the collection must exceed the payload limit");
    assert_limit_exceeded(error, JsonResource::PayloadBytes);

    checker
        .check_value(&Value::Bool(false))
        .expect("a failed collection check must restore the previous payload count");
}

#[test]
fn test_check_container_failure_does_not_mutate_accumulated_state() {
    let limits = JsonEncodeLimits::builder().max_output_bytes(5_usize).build();
    let mut checker = ValueWireEncodePreflight::new(limits);

    checker
        .check_value(&Value::Bool(true))
        .expect("the initial output byte must fit");
    let error = checker
        .check_container(&ValueContainer::from(vec![1_i32, 2]))
        .expect_err("the collection container must exceed the output limit");
    assert_limit_exceeded(error, JsonResource::OutputBytes);

    checker
        .check_value(&Value::Bool(false))
        .expect("a failed container check must restore the previous output count");
}

#[test]
fn test_check_container_covers_scalar_and_collection() {
    let limits = JsonEncodeLimits::builder().max_nodes(4_usize).build();
    let mut checker = ValueWireEncodePreflight::new(limits);

    checker
        .check_container(&ValueContainer::from(42_i32))
        .expect("a scalar container must fit");
    checker
        .check_container(&ValueContainer::from(vec![1_i32, 2]))
        .expect("a collection container must fit cumulatively");
}

#[test]
fn test_check_values_covers_empty_and_nested_json() {
    let mut checker = ValueWireEncodePreflight::new(JsonEncodeLimits::new());

    checker
        .check_values(&MultiValues::Int32(Vec::new()))
        .expect("an empty collection must be accepted");
    checker
        .check_values(&MultiValues::Json(vec![serde_json::json!({
            "items": [null, {"ready": true}]
        })]))
        .expect("nested JSON values must be traversed");
}

#[test]
fn test_preflight_handles_deep_json_without_recursion() {
    use crate::deep_json_helpers::run_isolated_test;

    let baseline = run_isolated_test(
        "value_wire::value_wire_encode_preflight_tests::test_preflight_deep_json_baseline_child",
        DEEP_PREFLIGHT_CHILD_ENV,
    );
    let baseline_log = String::from_utf8_lossy(&baseline.stderr);
    assert!(baseline_log.contains("checking"), "baseline log: {baseline_log}");
    assert!(
        baseline_log.contains("checked"),
        "preflight did not complete: {baseline_log}"
    );
    let baseline_stage = if baseline_log.contains("done") {
        "done"
    } else if baseline_log.contains("dropping") {
        "dropping"
    } else {
        "checked"
    };
    println!(
        "RV-007 baseline: rustc={}, os={}, status={:?}, last_stage={baseline_stage}",
        rustc_version(),
        std::env::consts::OS,
        baseline.status,
    );

    let iterative = run_isolated_test(
        "value_wire::value_wire_encode_preflight_tests::test_preflight_deep_json_iterative_child",
        DEEP_PREFLIGHT_CHILD_ENV,
    );
    let iterative_log = String::from_utf8_lossy(&iterative.stderr);
    let iterative_stage = if iterative_log.contains("done") {
        "done"
    } else if iterative_log.contains("iterative teardown fixture completed") {
        "iterative teardown fixture completed"
    } else if iterative_log.contains("checked") {
        "checked"
    } else {
        "no stage marker"
    };
    println!(
        "RV-007 iterative: rustc={}, os={}, status={:?}, last_stage={iterative_stage}",
        rustc_version(),
        std::env::consts::OS,
        iterative.status,
    );
    assert!(
        iterative.status.success(),
        "iterative preflight teardown child failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
        iterative.status,
        String::from_utf8_lossy(&iterative.stdout),
        String::from_utf8_lossy(&iterative.stderr),
    );
    assert!(
        String::from_utf8_lossy(&iterative.stderr).contains("done"),
        "iterative child did not complete all stages: {}",
        String::from_utf8_lossy(&iterative.stderr),
    );
}

#[test]
#[ignore = "run only through test_preflight_handles_deep_json_without_recursion"]
fn test_preflight_deep_json_baseline_child() {
    assert!(std::env::var_os(DEEP_PREFLIGHT_CHILD_ENV).is_some());
    let value = ManuallyDrop::new(Value::from_json_value(crate::deep_json_helpers::build_deep_json(
        10_000,
    )));
    eprintln!("checking");
    ValueWireEncodePreflight::new(JsonEncodeLimits::builder().max_depth(10_001).build())
        .check_value(&value)
        .expect("deep JSON preflight should succeed before ordinary drop");
    eprintln!("checked");
    let value = ManuallyDrop::into_inner(value);
    eprintln!("dropping");
    drop(value);
    eprintln!("done");
}

#[test]
#[ignore = "run only through test_preflight_handles_deep_json_without_recursion"]
fn test_preflight_deep_json_iterative_child() {
    assert!(std::env::var_os(DEEP_PREFLIGHT_CHILD_ENV).is_some());
    let value = ManuallyDrop::new(Value::from_json_value(crate::deep_json_helpers::build_deep_json(
        10_000,
    )));
    eprintln!("checking");
    let check_succeeded = ValueWireEncodePreflight::new(JsonEncodeLimits::builder().max_depth(10_001).build())
        .check_value(&value)
        .is_ok();
    eprintln!("checked");
    crate::deep_json_helpers::dismantle_json(crate::deep_json_helpers::build_deep_json(10_000));
    eprintln!("iterative teardown fixture completed");
    // `Value` has no safe consuming JSON getter. Keep the checked input in
    // `ManuallyDrop`; the isolated child process reclaims it when it exits.
    assert!(check_succeeded, "deep JSON preflight rejected the configured budget");
    eprintln!("done");
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|_| "unavailable".to_owned())
}
