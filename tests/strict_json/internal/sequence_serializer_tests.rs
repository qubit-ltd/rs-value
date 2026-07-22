// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests strict JSON sequence serialization through the public API.

use qubit_value::Value;

/// Verifies JSON sequences preserve element order.
#[test]
fn test_sequence_serializer_preserves_element_order() {
    let value = Value::Json(serde_json::json!([1, 2, 3]));
    assert_eq!(value.to_json_value().unwrap(), serde_json::json!([1, 2, 3]));
}
