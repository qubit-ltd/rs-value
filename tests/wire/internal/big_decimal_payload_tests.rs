// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Big-decimal V1 payload behavior.

#[cfg(feature = "big-decimal")]
#[test]
fn test_big_decimal_wire_rejects_excessive_scale() {

    assert!(crate::decode_value_wire_value(serde_json::json!({"version": 1, "value": {"scalar": {"bigdecimal": {"coefficient": "1", "scale": 150001}}}})).is_err());
}
