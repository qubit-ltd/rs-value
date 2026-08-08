// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Big-integer decimal wire behavior.

#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_round_trip() {
    use num_bigint::BigInt;
    use qubit_value::Value;
    use qubit_value::ValueWireV1;

    let expected = Value::BigInteger(BigInt::from(42));
    let wire = ValueWireV1::try_from(expected.clone()).unwrap();
    assert_eq!(
        serde_json::from_value::<ValueWireV1>(
            serde_json::to_value(wire).unwrap()
        )
        .unwrap()
        .into_container(),
        expected.into()
    );
}
