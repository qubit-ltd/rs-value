// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Finite-float adapter behavior.

use qubit_value::Value;
use qubit_value::ValueWireV1;

#[test]
fn test_float_wire_accepts_finite_value() {
    serde_json::to_value(ValueWireV1::try_from(Value::Float64(1.25)).unwrap())
        .expect("finite float wire value should serialize");
}
