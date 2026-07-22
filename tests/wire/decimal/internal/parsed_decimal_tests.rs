// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests parsed decimal validation.

use qubit_value::Value;

/// Verifies malformed decimal coefficients are rejected.
#[test]
fn test_parsed_decimal_rejects_malformed_coefficient() {
    let result = serde_json::from_value::<Value>(serde_json::json!({"bigdecimal": {"coefficient": "bad", "scale": 2}}));
    assert!(result.is_err());
}
