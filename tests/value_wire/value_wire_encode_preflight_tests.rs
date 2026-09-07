// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Contract tests for conservative Wire V1 encoding preflight checks.

use std::collections::HashMap;

use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodePreflight;

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
        .max_payload_bytes(4_usize)
        .build();
    let mut checker = ValueWireEncodePreflight::new(limits);

    checker
        .check_value(&Value::Bool(true))
        .expect("the initial payload byte must fit");
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
