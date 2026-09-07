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

    assert_eq!(
        crate::decode_value_wire_value(
            serde_json::json!({"version": 1, "value": {"scalar": {"uint128": "1"}}})
        )
        .unwrap()
        .into_container(),
        Value::UInt128(1).into(),
    );
}

/// Verifies collection parsing preserves order and rejects non-canonical text.
#[test]
fn test_wide_integer_wire_parses_collection_and_rejects_noncanonical_item() {
    use qubit_value::MultiValues;
    use qubit_value::Value;

    let decoded = crate::decode_value_wire_value(serde_json::json!({
        "version": 1,
        "value": {"collection": {"int128": ["-2", "0", "3"]}},
    }))
    .expect("canonical integer collection must decode")
    .into_container();
    assert_eq!(decoded, MultiValues::Int128(vec![-2, 0, 3]).into());

    let error = crate::decode_value_wire_value(serde_json::json!({
        "version": 1,
        "value": {"collection": {"int128": ["-2", "03"]}},
    }))
    .expect_err("non-canonical collection item must be rejected");
    assert!(
        error.to_string().contains("canonical"),
        "unexpected error: {error}"
    );
}
