// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests decimal wire payloads.

use qubit_value::Value;

/// Verifies decimal values serialize with their canonical wire payload.
#[test]
fn test_decimal_wire_serializes_canonical_payload() {
    let value = Value::BigDecimal("12.30".parse().unwrap());
    assert_eq!(
        serde_json::to_value(value).unwrap()["value"]["scalar"]["bigdecimal"]["scale"],
        serde_json::json!(2)
    );
}
