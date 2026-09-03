// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io;
use std::io::Write;

use qubit_budget::json::JsonResource;
use qubit_datatype::DataType;
use qubit_json::encode::JsonEncodeError;
use qubit_json::encode::JsonEncoder;
use qubit_json::encode::JsonIntegerSignedness;
use qubit_json::encode::JsonSerializationErrorKind;
use qubit_value::Value;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWirePayloadV1;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;

struct InvalidRawValue;

impl Serialize for InvalidRawValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let token = concat!("$", "serde_json", ":", ":private::RawValue");
        let mut state = serializer.serialize_struct(token, 1)?;
        state.serialize_field(token, "[")?;
        state.end()
    }
}

struct RejectingWriter;

impl Write for RejectingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("rejected"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Verifies V1 construction rejects float values JSON cannot represent.
#[test]
fn test_value_wire_encode_error_rejects_non_finite_float() {
    let result = ValueWirePayloadV1::try_from(Value::Float64(f64::NAN));

    assert!(matches!(
        result,
        Err(ValueWireEncodeError::NonFiniteFloat {
            data_type: DataType::Float64,
        })
    ));
}

/// Verifies wire conversion preserves the shared structured serialization
/// failure without introducing backend diagnostic text.
#[test]
fn test_value_wire_encode_error_preserves_serialization_kind() {
    let source = JsonEncoder::unlimited()
        .to_vec(&u128::MAX)
        .expect_err("wide integer must fail JSON serialization");
    let error = ValueWireEncodeError::from(source);

    assert!(matches!(
        error,
        ValueWireEncodeError::Json(source)
            if source.kind() == JsonSerializationErrorKind::IntegerOutOfRange {
                signedness: JsonIntegerSignedness::Unsigned,
            }
    ));
}

/// Verifies every non-budget owned encoder source keeps its wire-error shape.
#[test]
fn test_value_wire_encode_error_maps_owned_encoder_sources() {
    let invalid_raw: JsonEncodeError<JsonResource> = JsonEncoder::unlimited()
        .to_vec(&InvalidRawValue)
        .expect_err("invalid RawValue text must fail");
    let writer: JsonEncodeError<JsonResource> = JsonEncoder::unlimited()
        .write_buffered(RejectingWriter, &true)
        .expect_err("rejecting writer must fail");

    assert!(matches!(
        ValueWireEncodeError::from(invalid_raw),
        ValueWireEncodeError::Syntax(_)
    ));
    assert!(matches!(
        ValueWireEncodeError::from(writer),
        ValueWireEncodeError::Io(_)
    ));
}
