// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests decimal wire display formatting.

use qubit_value::Value;

/// Verifies decimal wire payloads preserve their coefficient text.
#[test]
fn test_display_decimal_preserves_coefficient_text() {
    let value = Value::BigDecimal("12.30".parse().unwrap());
    assert_eq!(serde_json::to_value(value).unwrap()["bigdecimal"]["coefficient"], serde_json::json!("1230"));
}
