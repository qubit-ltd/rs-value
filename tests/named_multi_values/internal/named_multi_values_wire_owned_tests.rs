// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies named collection deserialization through the owned wire DTO.

use qubit_value::{MultiValues, NamedMultiValues};

/// Deserializes a named collection from the V1 envelope.
#[test]
fn test_named_multi_values_wire_owned_deserializes_v1_envelope() {
    let decoded: NamedMultiValues = serde_json::from_value(serde_json::json!({
        "name": "ports",
        "value": {"version": 1, "value": {"collection": {"int32": [42]}}},
    }))
    .expect("named collection should deserialize");

    assert_eq!(
        decoded,
        NamedMultiValues::new("ports", MultiValues::Int32(vec![42]))
    );
}
