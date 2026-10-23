// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public finite-float wire behavior.

use qubit_value::Value;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWireV1;

#[test]
fn test_finite_float_wire_contract() {
    assert!(
        matches!(
            ValueWireV1::try_from(Value::Float64(f64::NAN)),
            Err(ValueWireEncodeError::NonFiniteFloat { .. })
        ),
        "wire conversion should reject non-finite floats"
    );
}

#[test]
fn test_float_wire_accepts_finite_value() {
    let wire = ValueWireV1::try_from(Value::Float64(1.25)).expect("finite float wire value should construct");

    serde_json::to_value(wire).expect("finite float wire value should serialize");
}
