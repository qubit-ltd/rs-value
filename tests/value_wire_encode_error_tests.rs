// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
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
