// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests borrowed wire shapes.

use qubit_value::ValueWireV1;

/// Verifies a scalar wire shape deserializes.
#[test]
fn test_wire_shape_ref_accepts_scalar_shape() {
    let value =
        serde_json::json!({"version": 1, "value": {"scalar": {"int32": 7}}});
    assert!(serde_json::from_value::<ValueWireV1>(value).is_ok());
}
