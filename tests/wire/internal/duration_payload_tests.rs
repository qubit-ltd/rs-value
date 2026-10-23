// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Duration V1 payload behavior.

use serde_json::json;

#[test]
fn test_duration_wire_rejects_invalid_nanos() {
    let input = json!({
        "version": 1,
        "value": {"scalar": {"duration": {"secs": 1, "nanos": 1_000_000_000}}},
    });

    assert!(
        crate::decode_value_wire_value(input).is_err(),
        "the decoder should reject duration payloads with nanoseconds outside the valid range",
    );
}
