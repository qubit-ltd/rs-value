// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests wide integer display formatting.

use qubit_value::Value;

/// Verifies wide integers display as decimal text in JSON.
#[test]
fn test_display_integer_uses_decimal_text() {
    assert_eq!(
        Value::UInt128(u128::MAX).to_json_value().unwrap(),
        serde_json::json!(u128::MAX.to_string())
    );
}
