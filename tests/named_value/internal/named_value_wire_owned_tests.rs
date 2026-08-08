// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies named scalar deserialization through the owned wire DTO.

use qubit_value::NamedValue;
use qubit_value::Value;

/// Deserializes a named scalar from the V1 envelope.
#[test]
fn test_named_value_wire_owned_deserializes_v1_envelope() {
    let decoded: NamedValue = serde_json::from_value(serde_json::json!({
        "name": "port",
        "value": {"version": 1, "value": {"scalar": {"int32": 42}}},
    }))
    .expect("named value should deserialize");

    assert_eq!(decoded, NamedValue::new("port", Value::Int32(42)));
}
