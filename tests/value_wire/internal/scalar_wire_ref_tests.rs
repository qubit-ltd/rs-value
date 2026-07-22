// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests borrowed scalar wire conversion.

use qubit_value::{
    Value,
    ValueWireV1,
};

/// Verifies scalar wire values deserialize without changing value type.
#[test]
fn test_scalar_wire_ref_deserializes_scalar() {
    let wire = ValueWireV1::from(Value::Int32(7));
    let decoded: ValueWireV1 =
        serde_json::from_value(serde_json::to_value(wire).unwrap()).unwrap();
    assert_eq!(decoded, ValueWireV1::from(Value::Int32(7)));
}
