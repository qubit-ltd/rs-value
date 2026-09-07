// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Wide-integer wire display behavior.

#[test]
fn test_wide_integer_wire_displays_uint128() {
    use qubit_value::Value;
    use qubit_value::ValueWireV1;

    assert_eq!(
        serde_json::to_value(ValueWireV1::try_from(Value::UInt128(1)).unwrap()).unwrap()["value"]["scalar"]
            ["uint128"],
        "1"
    );
}

/// Verifies serialization preserves the exact signed and unsigned limits.
#[test]
fn test_wide_integer_wire_displays_extreme_values() {
    use qubit_value::Value;
    use qubit_value::ValueWireV1;

    let signed = serde_json::to_value(ValueWireV1::try_from(Value::Int128(i128::MIN)).unwrap())
        .expect("serialize signed limit");
    let unsigned = serde_json::to_value(ValueWireV1::try_from(Value::UInt128(u128::MAX)).unwrap())
        .expect("serialize unsigned limit");

    assert_eq!(signed["value"]["scalar"]["int128"], i128::MIN.to_string());
    assert_eq!(
        unsigned["value"]["scalar"]["uint128"],
        u128::MAX.to_string()
    );
}
