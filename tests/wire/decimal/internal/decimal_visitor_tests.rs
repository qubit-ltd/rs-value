// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests decimal wire parsing.

use qubit_value::Value;

/// Verifies a canonical decimal wire payload deserializes.
#[test]
fn test_decimal_visitor_accepts_canonical_payload() {
    let value: Value = serde_json::from_value(serde_json::json!({"bigdecimal": {"coefficient": "1230", "scale": 2}})).unwrap();
    assert_eq!(value, Value::BigDecimal("12.30".parse().unwrap()));
}
