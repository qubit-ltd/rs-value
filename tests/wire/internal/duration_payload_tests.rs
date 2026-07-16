// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_value::Value;
use serde_json::json;

#[test]
fn test_duration_wire_payload_rejects_invalid_nanoseconds() {
    assert!(
        serde_json::from_value::<Value>(json!({
            "version": 1,
            "value": {
                "scalar": {
                    "duration": {"secs": 1, "nanos": 1_000_000_000_u64}
                }
            }
        }))
        .is_err()
    );
}
