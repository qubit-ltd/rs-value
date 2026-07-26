// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests wire data type tags.

use qubit_value::{
    Value,
    ValueWireV1,
};

/// Verifies the wire format uses the expected data type tag.
#[test]
fn test_wire_data_type_v1_writes_int32_tag() {
    let wire = ValueWireV1::from(Value::Int32(7));
    assert!(
        serde_json::to_value(wire).unwrap()["value"]["scalar"]
            .get("int32")
            .is_some()
    );
}
