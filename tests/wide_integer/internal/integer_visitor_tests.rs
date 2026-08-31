// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Wide-integer wire visitor behavior.

#[test]
fn test_wide_integer_wire_rejects_number_payload() {

    assert!(
        crate::decode_value_wire_value(serde_json::json!({"version": 1, "value": {"scalar": {"int128": 1}}}))
            .is_err()
    );
}
