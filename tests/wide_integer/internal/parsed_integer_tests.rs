// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Wide-integer wire parsing behavior.

#[test]
fn test_wide_integer_wire_parses_canonical_string() {
    use qubit_value::Value;
    use qubit_value::ValueWireV1;

    assert_eq!(
        serde_json::from_value::<ValueWireV1>(serde_json::json!({"version": 1, "value": {"scalar": {"uint128": "1"}}}))
            .unwrap()
            .into_container(),
        Value::UInt128(1).into(),
    );
}
