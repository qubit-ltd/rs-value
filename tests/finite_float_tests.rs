// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public finite-float wire behavior.

use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
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

#[test]
fn test_finite_float_collection_wire_round_trip() {
    let cases = [
        MultiValues::Float32(vec![0.0, -1.25, 3.5]),
        MultiValues::Float64(vec![0.0, -1.25, 3.5]),
    ];
    for values in cases {
        let expected = ValueContainer::Collection(values.clone());
        let wire = ValueWireV1::try_from(values).expect("finite collection must construct");
        let encoded = serde_json::to_string(&wire).expect("finite collection must encode");
        let restored = crate::decode_value_wire_str(&encoded).expect("finite collection must decode");
        assert_eq!(restored.into_container(), expected);
    }
}
