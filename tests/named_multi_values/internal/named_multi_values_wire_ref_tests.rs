// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies named collection serialization through the borrowed wire DTO.

use qubit_value::MultiValues;
use qubit_value::NamedMultiValues;

/// Serializes a named collection through the V1 envelope.
#[test]
fn test_named_multi_values_wire_ref_serializes_v1_envelope() {
    let named = NamedMultiValues::new("ports", MultiValues::Int32(vec![42]));

    assert_eq!(
        serde_json::to_value(named).expect("named collection should serialize"),
        serde_json::json!({
            "name": "ports",
            "value": {"version": 1, "value": {"collection": {"int32": [42]}}},
        }),
    );
}
