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
    use qubit_value::Value;

    assert_eq!(
        crate::decode_value_wire_value(serde_json::json!({"version": 1, "value": {"scalar": {"biginteger": "42"}}}))
            .unwrap()
            .into_container(),
        Value::BigInteger(42.into()).into()
    );
}

/// Verifies canonical decimal collections preserve order and exact values.
#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_decodes_extreme_collection() {
    use num_bigint::BigInt;
    use qubit_value::MultiValues;

    let values = vec![BigInt::from(-1), BigInt::from(0), BigInt::from(1)];
    let decoded = crate::decode_value_wire_value(serde_json::json!({
        "version": 1,
        "value": {"collection": {"biginteger": ["-1", "0", "1"]}},
    }))
    .expect("canonical big integer collection must decode")
    .into_container();

    assert_eq!(decoded, MultiValues::BigInteger(values).into());
}

/// Verifies a non-canonical collection element reports a parse failure.
#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_rejects_noncanonical_collection_item() {
    let error = crate::decode_value_wire_value(serde_json::json!({
        "version": 1,
        "value": {"collection": {"biginteger": ["1", "02"]}},
    }))
    .expect_err("non-canonical collection item must be rejected");

    assert!(error.to_string().contains("canonical"), "unexpected error: {error}");
}
