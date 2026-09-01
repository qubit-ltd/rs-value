// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_json::encode::JsonEncoder;
use qubit_json::encode::JsonSerializationErrorKind;
use qubit_value::Value;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWirePayloadV1;

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
                signedness: qubit_json::encode::JsonIntegerSignedness::Unsigned,
            }
    ));
}
