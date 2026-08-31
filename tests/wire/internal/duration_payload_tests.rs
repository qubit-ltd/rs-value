// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Duration V1 payload behavior.

#[test]
fn test_duration_wire_rejects_invalid_nanos() {

    assert!(
        crate::decode_value_wire_value(
            serde_json::json!({"version": 1, "value": {"scalar": {"duration": {"secs": 1, "nanos": 1_000_000_000}}}})
        )
        .is_err()
    );
}
