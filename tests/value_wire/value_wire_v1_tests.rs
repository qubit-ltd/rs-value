// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Regression tests for bounded `ValueWireV1` JSON decoding.

use std::io;
use std::io::Write;

use qubit_budget::BudgetError;
use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonEncodeLimits;
use qubit_budget::JsonResource;
use qubit_value::ValueContainer;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWireRefV1;
use qubit_value::ValueWireV1;

use crate::json_budget_test_support_tests::JsonDecodeLimitsExt;
use crate::json_budget_test_support_tests::JsonEncodeLimitsExt;

/// Writer that rejects every write to verify error conversion.
struct FailingWriter;

impl Write for FailingWriter {
    /// Rejects the supplied bytes with a stable test error.
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "test writer"))
    }

    /// Does not flush because writes are always rejected.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_value_wire_v1_default_json_limits_are_stable() {
    let decode = ValueWireV1::default_json_decode_limits();
    let encode = ValueWireV1::default_json_encode_limits();
    let limits = decode.value_limits();

    assert_eq!(decode.max_input_bytes(), Some(1_048_576));
    assert_eq!(encode.max_output_bytes(), Some(1_048_576));
    assert_eq!(limits.max_depth(), Some(64));
    assert_eq!(limits.max_nodes(), Some(100_000));
    assert_eq!(limits.max_sequence_items(), Some(4_096));
    assert_eq!(limits.max_map_entries(), Some(4_096));
    assert_eq!(limits.max_key_bytes(), Some(256 * 1024));
    assert_eq!(limits.max_string_bytes(), Some(256 * 1024));
    assert_eq!(limits.max_number_bytes(), Some(4_096));
}

#[test]
fn test_value_wire_v1_decode_json_slice_round_trips_with_default_limits() {
    let expected =
        ValueWireV1::try_from(ValueContainer::from(vec![1_i32, 2, 3])).expect("construct V1 wire");
    let input = serde_json::to_vec(&expected).expect("wire value should serialize");

    let actual = ValueWireV1::decode_json_slice(&input).expect("bounded input should decode");

    assert_eq!(actual, expected);
}

#[test]
fn test_value_wire_v1_decode_json_slice_honors_custom_limit() {
    let expected = ValueWireV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 wire");
    let input = serde_json::to_vec(&expected).expect("wire value should serialize");
    let limits = JsonDecodeLimits::default()
        .with_max_input_bytes(u64::try_from(input.len()).expect("input length must fit"));

    let actual = ValueWireV1::decode_json_slice_with_limits(&input, limits)
        .expect("input at the byte limit should decode");

    assert_eq!(actual, expected);
}

#[test]
fn test_value_wire_v1_rejects_oversized_input_before_parsing() {
    let input = b"definitely not valid JSON";
    let limits = JsonDecodeLimits::default()
        .with_max_input_bytes(u64::try_from(input.len() - 1).expect("input length must fit"));

    let error = ValueWireV1::decode_json_slice_with_limits(input, limits)
        .expect_err("oversized input must be rejected before JSON parsing");

    assert!(matches!(
        error,
        ValueWireDecodeError::Budget(
            BudgetError::LimitExceeded {
                resource: JsonResource::InputBytes,
                ..
            } | BudgetError::Insufficient {
                resource: JsonResource::InputBytes,
                ..
            }
        )
    ));
}

#[test]
fn test_value_wire_v1_reports_malformed_bounded_input() {
    let input = b"not JSON";
    let limits = JsonDecodeLimits::default()
        .with_max_input_bytes(u64::try_from(input.len()).expect("input length must fit"));

    let error = ValueWireV1::decode_json_slice_with_limits(input, limits)
        .expect_err("malformed bounded input must fail");

    assert!(matches!(error, ValueWireDecodeError::InvalidJson(_)));
}

#[test]
fn test_bounded_decode_reports_unsupported_version() {
    let error = ValueWireV1::decode_json_slice(br#"{"version":2,"value":{"scalar":{"int32":1}}}"#)
        .expect_err("version two must be rejected");

    assert!(matches!(
        error,
        ValueWireDecodeError::UnsupportedVersion {
            expected: 1,
            actual: 2
        }
    ));
}

#[test]
fn test_bounded_decode_reports_out_of_range_version_as_invalid_json() {
    let error =
        ValueWireV1::decode_json_slice(br#"{"version":256,"value":{"scalar":{"int32":1}}}"#)
            .expect_err("a version outside u8 must be rejected during typed decoding");

    assert!(matches!(error, ValueWireDecodeError::InvalidJson(_)));
}

#[test]
fn test_bounded_decode_reports_missing_version_as_invalid_json() {
    let error = ValueWireV1::decode_json_slice(br#"{"value":{"scalar":{"int32":1}}}"#)
        .expect_err("an envelope without a version must be rejected");

    assert!(matches!(error, ValueWireDecodeError::InvalidJson(_)));
}

#[test]
fn test_value_wire_v1_bounded_encoding_reports_budget_source() {
    let wire = ValueWireV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 wire");
    let error = wire
        .to_json_vec_with_limits(JsonEncodeLimits::default().with_max_output_bytes(1))
        .expect_err("the encoded wire should exceed one byte");

    assert!(matches!(
        error,
        ValueWireEncodeError::Budget(
            BudgetError::LimitExceeded {
                resource: JsonResource::OutputBytes,
                ..
            } | BudgetError::Insufficient {
                resource: JsonResource::OutputBytes,
                ..
            }
        )
    ));
}

#[test]
fn test_value_wire_v1_default_encoding_round_trips() {
    let wire = ValueWireV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 wire");
    let encoded = wire.to_json_vec().expect("default limits should encode");

    assert_eq!(
        encoded,
        serde_json::to_vec(&wire).expect("wire should serialize")
    );
    assert_eq!(
        ValueWireV1::decode_json_slice(&encoded).expect("default limits should decode"),
        wire
    );
}

#[test]
fn test_value_wire_v1_default_writer_encoding_matches_vec() {
    let wire = ValueWireV1::try_from(ValueContainer::from("ready")).expect("construct V1 wire");
    let mut output = Vec::new();

    wire.to_json_writer(&mut output)
        .expect("default limits should encode to writer");

    assert_eq!(
        output,
        wire.to_json_vec().expect("default limits should encode")
    );
}

#[test]
fn test_value_wire_ref_v1_bounded_encoding_matches_owned_wire() {
    let value = ValueContainer::from(42_i32);
    let owned = ValueWireV1::try_from(value.clone()).expect("construct V1 wire");
    let borrowed = ValueWireRefV1::try_from(&value).expect("construct borrowed V1 wire");

    assert_eq!(
        borrowed
            .to_json_vec_with_limits(ValueWireV1::default_json_encode_limits())
            .expect("borrowed wire should encode"),
        owned.to_json_vec().expect("owned wire should encode")
    );
}

#[test]
fn test_value_wire_v1_encoding_honors_structural_budgets() {
    let scalar = ValueWireV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 wire");
    let collection = ValueWireV1::try_from(ValueContainer::from(vec![1_i32, 2]))
        .expect("construct collection wire");
    let cases = [
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::default().with_max_depth(1)),
            JsonResource::Depth,
        ),
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::default().with_max_nodes(1)),
            JsonResource::Nodes,
        ),
        (
            collection
                .to_json_vec_with_limits(JsonEncodeLimits::default().with_max_sequence_items(1)),
            JsonResource::SequenceItems,
        ),
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::default().with_max_map_entries(1)),
            JsonResource::MapEntries,
        ),
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::default().with_max_key_bytes(1)),
            JsonResource::KeyBytes,
        ),
        (
            ValueWireV1::try_from(ValueContainer::from("ready"))
                .expect("construct string wire")
                .to_json_vec_with_limits(JsonEncodeLimits::default().with_max_string_bytes(1)),
            JsonResource::StringBytes,
        ),
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::default().with_max_number_bytes(0)),
            JsonResource::NumberBytes,
        ),
    ];

    for (result, resource) in cases {
        assert!(
            matches!(
                result,
                Err(ValueWireEncodeError::Budget(
                    BudgetError::LimitExceeded { resource: actual, .. }
                        | BudgetError::Insufficient { resource: actual, .. }
                )) if actual == resource
            ),
            "unexpected resource error: {result:?}, expected {resource:?}"
        );
    }
}

#[test]
fn test_value_wire_v1_writer_maps_io_errors() {
    let wire = ValueWireV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 wire");

    let error = wire
        .to_json_writer(FailingWriter)
        .expect_err("writer failure should be returned");

    assert!(matches!(error, ValueWireEncodeError::Io(_)));
}

#[test]
fn test_value_wire_v1_writer_budget_failure_does_not_write() {
    let wire = ValueWireV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 wire");
    let mut output = Vec::new();

    let error = wire
        .to_json_writer_with_limits(
            &mut output,
            JsonEncodeLimits::default().with_max_output_bytes(1),
        )
        .expect_err("output budget should reject the document");

    assert!(matches!(
        error,
        ValueWireEncodeError::Budget(
            BudgetError::LimitExceeded {
                resource: JsonResource::OutputBytes,
                ..
            } | BudgetError::Insufficient {
                resource: JsonResource::OutputBytes,
                ..
            }
        )
    ));
    assert!(output.is_empty());
}
