// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Big-integer decimal visitor behavior.

#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_rejects_noncanonical_string() {
    assert!(
        crate::decode_value_wire_value(serde_json::json!({"version": 1, "value": {"scalar": {"biginteger": "042"}}}))
            .is_err()
    );
}
