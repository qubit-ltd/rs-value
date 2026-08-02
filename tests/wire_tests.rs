// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests the public wire facade.

use qubit_value::{
    Value,
    ValueContainer,
    ValueWirePayloadV1,
    ValueWireV1,
};

/// Verifies the public wire facade preserves scalar values.
#[test]
fn test_wire_round_trips_scalar_value() {
    let wire =
        ValueWireV1::try_from(Value::Int32(7)).expect("construct V1 wire");
    let decoded: ValueWireV1 =
        serde_json::from_value(serde_json::to_value(wire).unwrap()).unwrap();
    assert_eq!(
        decoded,
        ValueWireV1::try_from(Value::Int32(7)).expect("construct V1 wire")
    );
}

/// Verifies nested protocols receive an unversioned but typed V1 payload.
#[test]
fn test_wire_payload_preserves_shape_without_version() {
    let payload =
        ValueWirePayloadV1::try_from(ValueContainer::from(vec![7_i32]))
            .expect("construct V1 payload");

    assert_eq!(
        serde_json::to_value(payload).expect("serialize V1 payload"),
        serde_json::json!({"collection": {"int32": [7]}}),
    );
}
