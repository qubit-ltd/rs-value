// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public finite-float wire behavior.

use qubit_value::{Value, ValueWireEncodeError, ValueWireV1};

#[test]
fn test_finite_float_wire_contract() {
    assert!(matches!(
        ValueWireV1::try_from(Value::Float64(f64::NAN)),
        Err(ValueWireEncodeError::NonFiniteFloat { .. })
    ));
}
