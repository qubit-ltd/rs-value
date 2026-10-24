// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed scalar V1 wire behavior.

use qubit_value::Value;
use qubit_value::ValueWireRefV1;
use serde_json::to_value;

#[test]
fn test_borrowed_scalar_wire_serializes() {
    let value = Value::Int32(1);
    let wire = ValueWireRefV1::try_from(&value).expect("construct borrowed scalar wire");

    to_value(wire).expect("serialize borrowed scalar wire");
}
