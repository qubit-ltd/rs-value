// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned V1 envelope behavior.

#[test]
fn test_owned_wire_envelope_requires_version_one() {
    use qubit_value::ValueWireV1;

    assert!(
        serde_json::from_value::<ValueWireV1>(serde_json::json!({"version": 2, "value": {"scalar": {"int32": 1}}}))
            .is_err()
    );
}
