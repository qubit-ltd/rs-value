// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Contract tests for conservative Wire V1 encoding preflight checks.

use std::collections::HashMap;
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
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;
use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWireV1;
use url::Url;

#[test]
fn test_preflight_accepts_encoded_uint128_with_small_number_limit() {
    let value = Value::UInt128(123);
    let limits = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_number_bytes(2)
        .build();
    let wire = ValueWireV1::try_from(value.clone()).unwrap();
    assert!(wire.to_json_vec_with_limits(limits).is_ok());
    assert!(
        ValueWireEncodePreflight::new(limits)
            .check_value(&value)
            .is_ok()
    );
}

#[test]
fn test_preflight_accepts_scientific_float_with_small_number_limit() {
    let value = Value::Float64(1e100);
    let limits = ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_number_bytes(10)
        .build();
    let wire = ValueWireV1::try_from(value.clone()).unwrap();
    assert!(wire.to_json_vec_with_limits(limits).is_ok());
    assert!(
        ValueWireEncodePreflight::new(limits)
            .check_value(&value)
            .is_ok()
    );
}

/// Verifies that an error identifies the expected exhausted JSON resource.
fn assert_limit_exceeded(
    error: MeasuredBudgetError<JsonResource, usize>,
    expected_resource: JsonResource,
) {
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
        (
            Value::BigDecimal("7.5".parse::<BigDecimal>().expect("decimal")),
            2,
        ),
        (Value::Duration(Duration::from_secs(1)), 1),
        (Value::Date(date), 1),
        (Value::Time(time), 1),
        (Value::DateTime(datetime), 1),
        (Value::Instant(instant), 1),
        (
            Value::Url(Url::parse("https://example.com/path").expect("URL")),
            1,
        ),
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
    let mut checker =
        ValueWireEncodePreflight::new(JsonEncodeLimits::builder().max_nodes(maximum_nodes).build());

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
        (
            MultiValues::String(vec!["a".to_owned(), "bc".to_owned()]),
            3,
        ),
        (
            MultiValues::StringMap(vec![HashMap::from([(
                "key".to_owned(),
                "value".to_owned(),
            )])]),
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
    let mut checker =
        ValueWireEncodePreflight::new(JsonEncodeLimits::builder().max_nodes(maximum_nodes).build());

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
            JsonEncodeLimits::builder()
                .max_string_bytes(2_usize)
                .build(),
            JsonResource::StringBytes,
            3,
            2,
        ),
        (
            Value::UInt64(123),
            JsonEncodeLimits::builder()
                .max_number_bytes(2_usize)
                .build(),
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

    let mut checker = ValueWireEncodePreflight::new(
        JsonEncodeLimits::builder()
            .max_sequence_items(1_usize)
            .build(),
    );
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
            JsonEncodeLimits::builder()
                .max_payload_bytes(0_usize)
                .build(),
            JsonResource::PayloadBytes,
            1,
        ),
        (
            JsonEncodeLimits::builder()
                .max_output_bytes(4_usize)
                .build(),
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
    let limits = JsonEncodeLimits::builder()
        .max_payload_bytes(1_usize)
        .build();
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
    let limits = JsonEncodeLimits::builder()
        .max_output_bytes(6_usize)
        .build();
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
