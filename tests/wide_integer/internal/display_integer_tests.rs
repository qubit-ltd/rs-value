// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Wide-integer wire display behavior.

use qubit_value::Value;
use qubit_value::ValueWireV1;
use serde_json::to_value;

#[test]
fn test_wide_integer_wire_displays_uint128() {
    let wire = ValueWireV1::try_from(Value::UInt128(1)).expect("encode unsigned integer");
    let json = to_value(wire).expect("serialize unsigned integer");

    assert_eq!(
        json["value"]["scalar"]["uint128"], "1",
        "unsigned integers must use canonical decimal strings"
    );
}

/// Verifies serialization preserves the exact signed and unsigned limits.
#[test]
fn test_wide_integer_wire_displays_extreme_values() {
    let signed_wire = ValueWireV1::try_from(Value::Int128(i128::MIN)).expect("encode signed limit");
    let signed = to_value(signed_wire).expect("serialize signed limit");
    let unsigned_wire = ValueWireV1::try_from(Value::UInt128(u128::MAX)).expect("encode unsigned limit");
    let unsigned = to_value(unsigned_wire).expect("serialize unsigned limit");

    assert_eq!(
        signed["value"]["scalar"]["int128"],
        i128::MIN.to_string(),
        "the signed minimum must retain its exact decimal representation"
    );
    assert_eq!(
        unsigned["value"]["scalar"]["uint128"],
        u128::MAX.to_string(),
        "the unsigned maximum must retain its exact decimal representation"
    );
}
