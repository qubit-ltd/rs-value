// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests borrowed wire envelopes.

use qubit_value::ValueWireV1;

/// Verifies an envelope with the supported version deserializes.
#[test]
fn test_wire_envelope_ref_accepts_supported_version() {
    let value = serde_json::json!({"version": 1, "value": {"scalar": {"int32": 7}}});
    assert!(serde_json::from_value::<ValueWireV1>(value).is_ok());
}
