// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Wide-integer wire visitor behavior.

#[test]
fn test_wide_integer_wire_rejects_number_payload() {
    assert!(
        crate::decode_value_wire_value(
            serde_json::json!({"version": 1, "value": {"scalar": {"int128": 1}}})
        )
        .is_err()
    );
}

/// Verifies malformed and non-canonical integer strings expose a parse error.
#[test]
fn test_wide_integer_wire_rejects_invalid_decimal_text() {
    for text in ["", "+1", "01", "12x", "-1"] {
        let input = serde_json::json!({
            "version": 1,
            "value": {"scalar": {"uint128": text}},
        });
        let error = crate::decode_value_wire_value(input)
            .expect_err("invalid unsigned integer text must be rejected");
        let message = error.to_string();
        if matches!(text, "+1" | "01") {
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

/// Verifies both signed and unsigned visitors accept their representable limits.
#[test]
fn test_wide_integer_wire_parses_extreme_values() {
    use qubit_value::Value;

    for (text, expected) in [
        (i128::MIN.to_string(), Value::Int128(i128::MIN).into()),
        (i128::MAX.to_string(), Value::Int128(i128::MAX).into()),
    ] {
        assert_eq!(
            crate::decode_value_wire_value(
                serde_json::json!({"version": 1, "value": {"scalar": {"int128": text}}})
            )
            .expect("signed limit must decode")
            .into_container(),
            expected,
        );
    }

    for (text, expected) in [
        (u128::MIN.to_string(), Value::UInt128(u128::MIN).into()),
        (u128::MAX.to_string(), Value::UInt128(u128::MAX).into()),
    ] {
        assert_eq!(
            crate::decode_value_wire_value(
                serde_json::json!({"version": 1, "value": {"scalar": {"uint128": text}}})
            )
            .expect("unsigned limit must decode")
            .into_container(),
            expected,
        );
    }
}
