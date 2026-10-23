// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed collection V1 wire behavior.

use qubit_value::ValueContainer;
use qubit_value::ValueWireRefV1;
use serde_json::json;
use serde_json::to_value;

#[test]
fn test_borrowed_collection_wire_serializes() {
    let value = ValueContainer::from(vec![1_i32]);
    let wire = ValueWireRefV1::try_from(&value).expect("construct borrowed collection wire");

    assert_eq!(
        to_value(wire).expect("serialize borrowed collection wire"),
        json!({
            "version": 1,
            "value": { "collection": { "int32": [1] } },
        }),
    );
}
