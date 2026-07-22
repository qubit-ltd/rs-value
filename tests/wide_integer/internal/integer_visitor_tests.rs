// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests wide integer JSON parsing.

use qubit_value::Value;

/// Verifies decimal JSON text deserializes as a wide integer wire payload.
#[test]
fn test_integer_visitor_accepts_decimal_text() {
    let value: Value = serde_json::from_value(serde_json::json!({
        "version": 1,
        "value": {"scalar": {"int128": i128::MAX.to_string()}},
    }))
    .unwrap();
    assert_eq!(value, Value::Int128(i128::MAX));
}
