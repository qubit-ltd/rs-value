// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Big-integer decimal wire behavior.

#[cfg(feature = "big-integer")]
use num_bigint::BigInt;
#[cfg(feature = "big-integer")]
use qubit_value::Value;
#[cfg(feature = "big-integer")]
use qubit_value::ValueWireV1;

#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_round_trip() {
    let expected = Value::BigInteger(BigInt::from(42));
    let wire = ValueWireV1::try_from(expected.clone()).expect("big integer converts to wire value");
    assert_eq!(
        crate::decode_value_wire_value(serde_json::to_value(wire).expect("wire value serializes"))
            .expect("wire value deserializes")
            .into_container(),
        expected.into()
    );
}
