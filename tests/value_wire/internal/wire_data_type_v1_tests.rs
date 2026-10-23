// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! V1 type-tag behavior.

use qubit_value::Value;
use qubit_value::ValueWireV1;

#[test]
fn test_wire_type_tag_is_decoded() {
    let wire_data = serde_json::json!({
        "version": 1,
        "value": {"scalar": {"int32": 1}},
    });
    let wire: ValueWireV1 = crate::decode_value_wire_value(wire_data).expect("valid V1 wire data should decode");

    assert_eq!(
        wire.into_container(),
        Value::Int32(1).into(),
        "the decoded value should preserve its int32 type tag",
    );
}
