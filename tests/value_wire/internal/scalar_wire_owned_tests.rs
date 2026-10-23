// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned scalar V1 wire behavior.

use qubit_value::Value;
use qubit_value::ValueWireV1;
use serde_json::to_value;

#[test]
fn test_owned_scalar_wire_round_trip() {
    let wire = ValueWireV1::try_from(Value::Int32(1)).expect("construct wire");
    assert_eq!(
        crate::decode_value_wire_value(to_value(wire).expect("wire value should serialize"))
            .expect("wire value should decode")
            .into_container(),
        Value::Int32(1).into()
    );
}
