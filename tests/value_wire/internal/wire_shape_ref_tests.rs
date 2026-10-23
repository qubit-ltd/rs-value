// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 shape behavior.

use qubit_value::MultiValues;
use qubit_value::ValueWireRefV1;
use serde_json::json;
use serde_json::to_value;

#[test]
fn test_borrowed_wire_shape_preserves_collection() {
    let wire = ValueWireRefV1::try_from(&MultiValues::Int32(vec![1])).unwrap();
    let value = to_value(wire).unwrap();

    assert_eq!(value["value"]["collection"]["int32"], json!([1]));
}
