// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! 128-bit integer wire behavior.

use qubit_value::Value;
use qubit_value::ValueWireV1;
use serde_json::to_value;

#[test]
fn test_wide_integer_wire_uses_canonical_string() {
    let wire_value = ValueWireV1::try_from(Value::Int128(-1))
        .expect("the negative 128-bit integer should have a wire representation");
    let json_value = to_value(wire_value).expect("the integer wire value should serialize to JSON");

    assert_eq!(
        json_value["value"]["scalar"]["int128"], "-1",
        "the wire payload should encode the integer as its canonical decimal string"
    );
}
