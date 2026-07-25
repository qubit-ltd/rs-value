// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests owned wire envelopes.

use qubit_value::{Value, ValueWireV1};

/// Verifies owned wire envelopes include the format version.
#[test]
fn test_wire_envelope_owned_writes_version() {
    let wire = ValueWireV1::from(Value::Int32(7));
    assert_eq!(
        serde_json::to_value(wire).unwrap()["version"],
        serde_json::json!(1)
    );
}
