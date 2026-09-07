// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Big-integer decimal visitor behavior.

#[cfg(feature = "big-integer")]
use qubit_value::Value;
#[cfg(feature = "big-integer")]
use qubit_value::ValueWireV1Seed;
#[cfg(feature = "big-integer")]
use serde::de::DeserializeSeed;
#[cfg(feature = "big-integer")]
use serde::de::IntoDeserializer;

#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_rejects_noncanonical_string() {
    assert!(
        crate::decode_value_wire_value(
            serde_json::json!({"version": 1, "value": {"scalar": {"biginteger": "042"}}})
        )
        .is_err()
    );
}

/// Verifies decimal visitors reject malformed and non-canonical integers.
#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_rejects_invalid_decimal_text() {
    for text in ["", "+1", "01", "-0", "12x"] {
        let input = serde_json::json!({
            "version": 1,
            "value": {"scalar": {"biginteger": text}},
        });
        let error = crate::decode_value_wire_value(input)
            .expect_err("invalid big integer text must be rejected");
        let message = error.to_string();
        if matches!(text, "+1" | "01" | "-0") {
            assert!(
                message.contains("canonical"),
                "unexpected error for {text:?}: {error}"
            );
        } else {
            assert!(
                message.contains("invalid") || message.contains("empty"),
                "unexpected parse error for {text:?}: {error}"
            );
        }
    }
}

/// Verifies the decimal visitor accepts an owned extreme-magnitude integer string.
#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_parses_owned_extreme_string() {
    let text = format!("-{}", "9".repeat(256));
    let expected = text
        .parse::<num_bigint::BigInt>()
        .expect("test integer must parse");
    let input = serde_json::json!({"version": 1, "value": {"scalar": {"biginteger": text}}});
    let wire = ValueWireV1Seed::new()
        .deserialize(input.into_deserializer())
        .expect("owned extreme integer string must decode");
    assert_eq!(wire.into_container(), Value::BigInteger(expected).into());
}

/// Verifies an owned non-canonical decimal string reports the exact visitor error.
#[cfg(feature = "big-integer")]
#[test]
fn test_big_integer_wire_rejects_owned_noncanonical_string_precisely() {
    let input = serde_json::json!({"version": 1, "value": {"scalar": {"biginteger": "+1"}}});
    let error = ValueWireV1Seed::new()
        .deserialize(input.into_deserializer())
        .expect_err("owned non-canonical integer string must be rejected");
    assert_eq!(error.to_string(), "non-canonical decimal string");
}
