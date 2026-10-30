// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Big-decimal V1 payload behavior.

#[cfg(feature = "big-decimal")]
use serde_json::json;

#[cfg(feature = "big-decimal")]
#[test]
fn test_big_decimal_wire_rejects_excessive_scale() {
    let input = json!({
        "version": 1,
        "value": {"scalar": {"bigdecimal": {"coefficient": "1", "scale": 150001}}},
    });

    assert!(
        crate::decode_value_wire_value(input).is_err(),
        "the decoder should reject a big-decimal payload with an excessive scale",
    );
}
