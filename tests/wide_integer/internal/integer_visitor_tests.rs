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
    use qubit_value::ValueWireV1;

    assert!(
        serde_json::from_value::<ValueWireV1>(serde_json::json!({"version": 1, "value": {"scalar": {"int128": 1}}}))
            .is_err()
    );
}
