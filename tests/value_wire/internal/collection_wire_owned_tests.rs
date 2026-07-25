// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests owned collection wire conversion.

use qubit_value::{MultiValues, ValueWireV1};

/// Verifies owned collections serialize through the versioned wire format.
#[test]
fn test_collection_wire_owned_serializes_collection() {
    let wire = ValueWireV1::from(MultiValues::Int32(vec![7]));
    assert_eq!(
        serde_json::to_value(wire).unwrap()["value"]["collection"]["int32"],
        serde_json::json!([7])
    );
}
