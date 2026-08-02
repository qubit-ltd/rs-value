// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Regression tests for bounded `ValueWireV1` JSON decoding.

use qubit_value::{
    ValueContainer,
    ValueWireDecodeError,
    ValueWireLimits,
    ValueWireV1,
};

#[test]
fn test_value_wire_v1_decode_json_slice_round_trips_with_default_limits() {
    let expected =
        ValueWireV1::try_from(ValueContainer::from(vec![1_i32, 2, 3]))
            .expect("construct V1 wire");
    let input =
        serde_json::to_vec(&expected).expect("wire value should serialize");

    let actual = ValueWireV1::decode_json_slice(&input)
        .expect("bounded input should decode");

    assert_eq!(actual, expected);
}

#[test]
fn test_value_wire_v1_decode_json_slice_honors_custom_limit() {
    let expected = ValueWireV1::try_from(ValueContainer::from(42_i32))
        .expect("construct V1 wire");
    let input =
        serde_json::to_vec(&expected).expect("wire value should serialize");
    let limits = ValueWireLimits::new(input.len());

    let actual = ValueWireV1::decode_json_slice_with_limits(&input, limits)
        .expect("input at the byte limit should decode");

    assert_eq!(actual, expected);
}

#[test]
fn test_value_wire_v1_rejects_oversized_input_before_parsing() {
    let input = b"definitely not valid JSON";
    let limits = ValueWireLimits::new(input.len() - 1);

    let error = ValueWireV1::decode_json_slice_with_limits(input, limits)
        .expect_err("oversized input must be rejected before JSON parsing");

    assert!(matches!(
        error,
        ValueWireDecodeError::InputTooLarge {
            input_bytes,
            max_input_bytes,
        } if input_bytes == input.len() && max_input_bytes == input.len() - 1
    ));
}

#[test]
fn test_value_wire_v1_reports_malformed_bounded_input() {
    let input = b"not JSON";
    let limits = ValueWireLimits::new(input.len());

    let error = ValueWireV1::decode_json_slice_with_limits(input, limits)
        .expect_err("malformed bounded input must fail");

    assert!(matches!(error, ValueWireDecodeError::InvalidJson(_)));
}
