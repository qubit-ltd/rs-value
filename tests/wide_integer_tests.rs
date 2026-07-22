// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests wide integer JSON behavior.

use qubit_value::Value;

/// Verifies 128-bit integers retain their decimal JSON representation.
#[test]
fn test_wide_integer_serializes_as_decimal_text() {
    assert_eq!(
        serde_json::to_value(Value::Int128(i128::MAX)).unwrap(),
        serde_json::json!({
            "version": 1,
            "value": {"scalar": {"int128": i128::MAX.to_string()}},
        })
    );
}
