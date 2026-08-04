// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for `ValueWireLimits`.

use qubit_value::{
    MultiValues,
    Value,
    ValueContainer,
    ValueWireDecodeError,
    ValueWireLimits,
    ValueWirePayloadV1,
    WireLimits,
};
#[cfg(feature = "big-decimal")]
use {
    bigdecimal::BigDecimal,
    num_bigint::BigInt,
};

#[test]
fn test_value_wire_limits_default_uses_documented_byte_budget() {
    let limits = ValueWireLimits::default();

    assert_eq!(
        limits.max_json_bytes(),
        ValueWireLimits::DEFAULT_MAX_JSON_BYTES
    );
    assert_eq!(limits.max_json_bytes(), 1_048_576);
}

#[test]
fn test_value_wire_limits_new_preserves_custom_byte_budget() {
    let limits = ValueWireLimits::new(64 * 1024);

    assert_eq!(limits.max_json_bytes(), 65_536);
}

#[test]
fn test_value_wire_limits_check_json_bytes_enforces_public_budget() {
    let limits = ValueWireLimits::new(8);

    limits
        .check_json_bytes(8)
        .expect("input at the byte budget should be accepted");
    assert!(matches!(
        limits.check_json_bytes(9),
        Err(ValueWireDecodeError::InputTooLarge {
            input_bytes: 9,
            max_input_bytes: 8,
        })
    ));
}

#[test]
fn test_wire_limits_reject_collection_items() {
    let payload = ValueWirePayloadV1::try_from(MultiValues::Int32(vec![1, 2]))
        .expect("the finite collection should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_collection_items(1);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::CollectionItems,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_wire_limits_reject_string_bytes() {
    let payload = ValueWirePayloadV1::try_from(ValueContainer::Scalar(
        Value::String("hello".to_owned()),
    ))
    .expect("the string should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_string_bytes(4);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::StringBytes,
            value: 5,
            maximum: 4,
        })
    ));
}

#[test]
fn test_wire_budget_accumulates_nodes_across_values() {
    let mut budget = WireLimits::new(0)
        .with_max_nodes(1)
        .begin(0)
        .expect("the empty input budget should start");

    budget
        .check_value(&Value::Int32(1))
        .expect("the first value should fit the node limit");
    assert!(matches!(
        budget.check_value(&Value::Int32(2)),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::Nodes,
            value: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_wire_limits_reject_numeric_payload_length() {
    let payload = ValueWirePayloadV1::try_from(Value::Int128(12_345))
        .expect("the integer should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_numeric_digits(4);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::NumericDigits,
            value: 5,
            maximum: 4,
        })
    ));
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_wire_budget_counts_big_decimal_coefficient_without_expanding_scale() {
    let value = Value::BigDecimal(BigDecimal::new(BigInt::from(1), -150_000));
    let mut budget = WireLimits::new(0)
        .with_max_numeric_digits(1)
        .begin(0)
        .expect("the empty input budget should start");

    budget
        .check_value(&value)
        .expect("the one-digit coefficient should fit without scale expansion");
}

#[test]
fn test_json_preflight_rejects_excessive_string_before_runtime_decode() {
    let payload = ValueWirePayloadV1::try_from(Value::String("x".repeat(65)))
        .expect("the string should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_string_bytes(4);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::StringBytes,
            value: 65,
            maximum: 64,
        })
    ));
}

#[test]
fn test_json_preflight_rejects_excessive_object_before_runtime_decode() {
    let value = Value::Json(serde_json::Value::Object(
        (0..17)
            .map(|index| (format!("key{index}"), serde_json::json!(index)))
            .collect(),
    ));
    let payload = ValueWirePayloadV1::try_from(value)
        .expect("the JSON object should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_map_entries(1);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::MapEntries,
            value: 17,
            maximum: 16,
        })
    ));
}

#[test]
fn test_json_preflight_rejects_excessive_nodes_before_runtime_decode() {
    let payload =
        ValueWirePayloadV1::try_from(MultiValues::Int32((0..70).collect()))
            .expect("the collection should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len())
        .with_max_nodes(1)
        .with_max_collection_items(70);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::Nodes,
            value: 66,
            maximum: 65,
        })
    ));
}

#[test]
fn test_json_preflight_rejects_excessive_depth_before_runtime_decode() {
    let mut json = serde_json::json!(0);
    for _ in 0..18 {
        json = serde_json::json!([json]);
    }
    let payload = ValueWirePayloadV1::try_from(Value::Json(json))
        .expect("the nested JSON should be wire-compatible");
    let input =
        serde_json::to_vec(&payload).expect("the payload should serialize");
    let limits = WireLimits::new(input.len()).with_max_depth(1);

    assert!(matches!(
        ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits),
        Err(ValueWireDecodeError::LimitExceeded {
            kind: qubit_value::ValueWireLimitKind::Depth,
            value: 18,
            maximum: 17,
        })
    ));
}
