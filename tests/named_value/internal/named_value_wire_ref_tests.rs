// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies named scalar serialization through the borrowed wire DTO.

use qubit_value::{
    NamedValue,
    Value,
};

/// Serializes a named scalar through the V1 envelope.
#[test]
fn test_named_value_wire_ref_serializes_v1_envelope() {
    let named = NamedValue::new("port", Value::Int32(42));

    assert_eq!(
        serde_json::to_value(named).expect("named value should serialize"),
        serde_json::json!({
            "name": "port",
            "value": {"version": 1, "value": {"scalar": {"int32": 42}}},
        }),
    );
}
