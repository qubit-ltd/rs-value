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
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::MultiValues;
use qubit_value::NamedMultiValues;
use qubit_value::NamedValue;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWireRefV1;
use qubit_value::ValueWireV1;

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

struct WriterProbe {
    bytes: Vec<u8>,
    remaining: Option<usize>,
    flush_calls: usize,
}

impl Write for WriterProbe {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.remaining == Some(0) {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "prefix accepted"));
        }
        let accepted = self.remaining.map_or(input.len(), |n| n.min(input.len()));
        self.bytes.extend_from_slice(&input[..accepted]);
        if let Some(remaining) = &mut self.remaining {
            *remaining -= accepted;
        }
        Ok(accepted)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_calls += 1;
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "explicit flush"))
    }
}

fn assert_partial_writer<F>(expected: &[u8], encode: F)
where
    F: FnOnce(&mut WriterProbe) -> Result<(), ValueWireEncodeError>,
{
    let mut writer = WriterProbe {
        bytes: Vec::new(),
        remaining: Some(3),
        flush_calls: 0,
    };
    let error = encode(&mut writer).expect_err("I/O failure must be returned");
    match error {
        ValueWireEncodeError::Io(source) => {
            assert_eq!(source.kind(), io::ErrorKind::BrokenPipe);
        }
        other => panic!("unexpected encoding error: {other:?}"),
    }
    assert_eq!(writer.bytes, expected[..3]);
    assert_eq!(writer.flush_calls, 0);
}

fn assert_writer_does_not_flush<F>(expected: &[u8], encode: F)
where
    F: FnOnce(&mut WriterProbe) -> Result<(), ValueWireEncodeError>,
{
    let mut writer = WriterProbe {
        bytes: Vec::new(),
        remaining: None,
        flush_calls: 0,
    };
    encode(&mut writer).expect("accepting all bytes completes encoding");
    assert_eq!(writer.bytes, expected);
    assert_eq!(writer.flush_calls, 0);
    assert_eq!(writer.flush().unwrap_err().kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(writer.flush_calls, 1);
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
    let expected = ValueWireV1::try_from(ValueContainer::from(vec![1_i32, 2, 3])).expect("construct V1 wire");
    let input = serde_json::to_vec(&expected).expect("wire value should serialize");

    let actual = ValueWireV1::decode_json_slice(&input).expect("bounded input should decode");

    assert_eq!(actual, expected);
}

#[test]
fn test_value_wire_v1_json_numbers_round_trip() {
    let expected = ValueWireV1::try_from(Value::Json(serde_json::json!({
        "amount": 1.5,
        "items": [100.0, -0.0],
        "marker": {"$serde_json::private::Number": "1.5"},
    })))
    .expect("construct JSON wire value");
    let encoded = expected.to_json_vec().expect("encode JSON wire value");
    let decoded = ValueWireV1::decode_json_slice(&encoded).expect("decode JSON wire value");

    assert_eq!(
        decoded, expected,
        "JSON numbers and literal marker objects must remain distinct"
    );
    assert_eq!(decoded.to_json_vec().expect("re-encode JSON wire value"), encoded);
}

#[test]
fn test_value_wire_v1_decode_json_slice_honors_custom_limit() {
    let expected = ValueWireV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 wire");
    let input = serde_json::to_vec(&expected).expect("wire value should serialize");
    let limits = JsonDecodeLimits::builder().max_input_bytes(input.len()).build();

    let actual =
        ValueWireV1::decode_json_slice_with_limits(&input, limits).expect("input at the byte limit should decode");

    assert_eq!(actual, expected);
}

#[test]
fn test_value_wire_v1_rejects_oversized_input_before_parsing() {
    let input = b"definitely not valid JSON";
    let limits = JsonDecodeLimits::builder().max_input_bytes(input.len() - 1).build();

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
fn test_value_wire_v1_reports_malformed_bounded_input_as_syntax() {
    let input = b"not JSON";
    let limits = JsonDecodeLimits::builder().max_input_bytes(input.len()).build();

    let error =
        ValueWireV1::decode_json_slice_with_limits(input, limits).expect_err("malformed bounded input must fail");

    assert!(matches!(error, ValueWireDecodeError::Syntax(_)));
}

#[test]
fn test_value_wire_v1_reports_typed_mismatch_as_invalid_json() {
    let error =
        ValueWireV1::decode_json_slice(br#"{"version":1,"value":false}"#).expect_err("a boolean is not a V1 payload");

    assert!(
        matches!(error, ValueWireDecodeError::InvalidJson(_)),
        "valid JSON with an invalid V1 DTO must be a typed decode error: {error:?}"
    );
}

#[test]
fn test_value_wire_v1_decoding_honors_structural_budgets() {
    let scalar = ValueWireV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 wire");
    let string = ValueWireV1::try_from(ValueContainer::from("ready")).expect("construct string wire");
    let scalar_input = serde_json::to_vec(&scalar).expect("scalar wire should serialize");
    let string_input = serde_json::to_vec(&string).expect("string wire should serialize");
    let cases = [
        (
            scalar_input.as_slice(),
            JsonDecodeLimits::builder().max_depth(1).build(),
            JsonResource::Depth,
        ),
        (
            scalar_input.as_slice(),
            JsonDecodeLimits::builder().max_nodes(1).build(),
            JsonResource::Nodes,
        ),
        (
            string_input.as_slice(),
            JsonDecodeLimits::builder().max_string_bytes(1).build(),
            JsonResource::StringBytes,
        ),
    ];

    for (input, limits, resource) in cases {
        let result = ValueWireV1::decode_json_slice_with_limits(input, limits);
        assert!(
            matches!(
                result,
                Err(ValueWireDecodeError::Budget(
                    BudgetError::LimitExceeded { resource: actual, .. }
                        | BudgetError::Insufficient { resource: actual, .. }
                )) if actual == resource
            ),
            "unexpected resource error: {result:?}, expected {resource:?}"
        );
    }
}

#[test]
fn test_bounded_decode_reports_unsupported_version() {
    let error = ValueWireV1::decode_json_slice(br#"{"version":2,"value":{"scalar":{"int32":1}}}"#)
        .expect_err("version two must be rejected");

    assert!(matches!(
        error,
        ValueWireDecodeError::UnsupportedVersion { expected: 1, actual: 2 }
    ));
}

#[test]
fn test_bounded_decode_reports_out_of_range_version_as_invalid_json() {
    let error = ValueWireV1::decode_json_slice(br#"{"version":256,"value":{"scalar":{"int32":1}}}"#)
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
        .to_json_vec_with_limits(JsonEncodeLimits::builder().max_output_bytes(1).build())
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

    assert_eq!(encoded, serde_json::to_vec(&wire).expect("wire should serialize"));
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

    assert_eq!(output, wire.to_json_vec().expect("default limits should encode"));
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
    let collection = ValueWireV1::try_from(ValueContainer::from(vec![1_i32, 2])).expect("construct collection wire");
    let cases = [
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::builder().max_depth(1).build()),
            JsonResource::Depth,
        ),
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::builder().max_nodes(1).build()),
            JsonResource::Nodes,
        ),
        (
            collection.to_json_vec_with_limits(JsonEncodeLimits::builder().max_sequence_items(1).build()),
            JsonResource::SequenceItems,
        ),
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::builder().max_map_entries(1).build()),
            JsonResource::MapEntries,
        ),
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::builder().max_key_bytes(1).build()),
            JsonResource::KeyBytes,
        ),
        (
            ValueWireV1::try_from(ValueContainer::from("ready"))
                .expect("construct string wire")
                .to_json_vec_with_limits(JsonEncodeLimits::builder().max_string_bytes(1).build()),
            JsonResource::StringBytes,
        ),
        (
            scalar.to_json_vec_with_limits(JsonEncodeLimits::builder().max_number_bytes(0).build()),
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
        .to_json_writer_with_limits(&mut output, JsonEncodeLimits::builder().max_output_bytes(1).build())
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

#[test]
fn test_public_wire_writers_preserve_io_and_flush_contracts() {
    let scalar = Value::new(42_i32);
    let values = MultiValues::new([1_i32, 2]);
    let owned = ValueWireV1::try_from(scalar.clone()).unwrap();
    let borrowed = ValueWireRefV1::from_value(&scalar).unwrap();
    let payload = ValueWirePayloadV1::try_from(scalar.clone()).unwrap();
    let payload_ref = ValueWirePayloadRefV1::from_value(&scalar).unwrap();
    let named = NamedValue::new("answer", scalar.clone());
    let named_values = NamedMultiValues::new("items", values);

    macro_rules! check_writer {
        ($dto:expr) => {{
            let dto = &$dto;
            let expected = dto.to_json_vec().unwrap();
            assert_partial_writer(&expected, |writer| dto.to_json_writer(writer));
            assert_partial_writer(&expected, |writer| {
                dto.to_json_writer_with_limits(writer, JsonEncodeLimits::builder().max_output_bytes(4096).build())
            });
            assert_writer_does_not_flush(&expected, |writer| dto.to_json_writer(writer));
            assert_writer_does_not_flush(&expected, |writer| {
                dto.to_json_writer_with_limits(writer, JsonEncodeLimits::builder().max_output_bytes(4096).build())
            });
        }};
    }

    check_writer!(owned);
    check_writer!(borrowed);
    check_writer!(payload);
    check_writer!(payload_ref);
    check_writer!(named);
    check_writer!(named_values);
}
