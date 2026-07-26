// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests borrowed collection wire conversion.

use qubit_value::{
    MultiValues,
    ValueWireV1,
};

/// Verifies collection wire values deserialize without changing shape.
#[test]
fn test_collection_wire_ref_deserializes_collection() {
    let wire = ValueWireV1::from(MultiValues::Int32(vec![7]));
    let decoded: ValueWireV1 =
        serde_json::from_value(serde_json::to_value(wire).unwrap()).unwrap();
    assert_eq!(decoded, ValueWireV1::from(MultiValues::Int32(vec![7])));
}
