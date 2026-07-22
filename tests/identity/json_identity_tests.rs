// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests JSON identity normalization.

use qubit_value::Value;

/// Verifies JSON object key order does not affect public value identity.
#[test]
fn test_json_identity_ignores_object_key_order() {
    let left = Value::Json(serde_json::json!({"first": 1, "second": 2}));
    let right = Value::Json(serde_json::json!({"second": 2, "first": 1}));
    assert_eq!(left, right);
}
