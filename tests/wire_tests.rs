// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests the public wire facade.

use qubit_value::{Value, ValueWireV1};

/// Verifies the public wire facade preserves scalar values.
#[test]
fn test_wire_round_trips_scalar_value() {
    let wire = ValueWireV1::from(Value::Int32(7));
    let decoded: ValueWireV1 = serde_json::from_value(serde_json::to_value(wire).unwrap()).unwrap();
    assert_eq!(decoded, ValueWireV1::from(Value::Int32(7)));
}
