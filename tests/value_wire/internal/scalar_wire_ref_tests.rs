// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed scalar V1 wire behavior.

#[test]
fn test_borrowed_scalar_wire_serializes() {
    use qubit_value::Value;
    use qubit_value::ValueWireRefV1;

    assert!(serde_json::to_value(ValueWireRefV1::try_from(&Value::Int32(1)).unwrap()).is_ok());
}
