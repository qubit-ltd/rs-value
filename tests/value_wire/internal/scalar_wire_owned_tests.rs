// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned scalar V1 wire behavior.

#[test]
fn test_owned_scalar_wire_round_trip() {
    use qubit_value::{Value, ValueWireV1};

    let wire = ValueWireV1::try_from(Value::Int32(1)).expect("construct wire");
    assert_eq!(
        serde_json::from_value::<ValueWireV1>(serde_json::to_value(wire).unwrap())
            .unwrap()
            .into_container(),
        Value::Int32(1).into()
    );
}
