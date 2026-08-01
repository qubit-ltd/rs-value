// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Big-integer decimal parsing behavior.

#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_decodes_canonical_string() {
    use qubit_value::{Value, ValueWireV1};

    assert_eq!(
        serde_json::from_value::<ValueWireV1>(
            serde_json::json!({"version": 1, "value": {"scalar": {"biginteger": "42"}}})
        )
        .unwrap()
        .into_container(),
        Value::BigInteger(42.into()).into()
    );
}
