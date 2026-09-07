// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Big-integer decimal display behavior.

#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_serializes_negative_value() {
    use num_bigint::BigInt;
    use qubit_value::Value;
    use qubit_value::ValueWireV1;

    assert_eq!(
        serde_json::to_value(ValueWireV1::try_from(Value::BigInteger(BigInt::from(-42))).unwrap())
            .unwrap()["value"]["scalar"]["biginteger"],
        "-42"
    );
}

/// Verifies decimal display preserves a large integer without numeric coercion.
#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_serializes_large_value_as_text() {
    use num_bigint::BigInt;
    use qubit_value::Value;
    use qubit_value::ValueWireV1;

    let value =
        BigInt::parse_bytes(b"123456789012345678901234567890", 10).expect("large integer fixture");
    let wire = serde_json::to_value(ValueWireV1::try_from(Value::BigInteger(value)).unwrap())
        .expect("serialize large integer");

    assert_eq!(
        wire["value"]["scalar"]["biginteger"],
        "123456789012345678901234567890"
    );
}
