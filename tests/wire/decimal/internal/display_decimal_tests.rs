// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Big-integer decimal display behavior.

#[cfg(feature = "big-integer")]
use num_bigint::BigInt;
#[cfg(feature = "big-integer")]
use qubit_value::Value;
#[cfg(feature = "big-integer")]
use qubit_value::ValueWireV1;
#[cfg(feature = "big-integer")]
use serde_json::to_value;

#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_serializes_negative_value() {
    let wire = to_value(
        ValueWireV1::try_from(Value::BigInteger(BigInt::from(-42))).expect("big integer converts to wire value"),
    )
    .expect("wire value serializes");

    assert_eq!(wire["value"]["scalar"]["biginteger"], "-42");
}

/// Verifies decimal display preserves a large integer without numeric coercion.
#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_serializes_large_value_as_text() {
    let value = BigInt::parse_bytes(b"123456789012345678901234567890", 10).expect("large integer fixture");
    let wire = to_value(ValueWireV1::try_from(Value::BigInteger(value)).expect("big integer converts to wire value"))
        .expect("serialize large integer");

    assert_eq!(wire["value"]["scalar"]["biginteger"], "123456789012345678901234567890");
}
