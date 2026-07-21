// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for `ValueWireDecodeError`.

use std::error::Error;

use qubit_value::ValueWireDecodeError;

#[test]
fn test_value_wire_decode_error_describes_input_limit() {
    let error = ValueWireDecodeError::InputTooLarge {
        input_bytes: 9,
        max_input_bytes: 8,
    };

    assert_eq!(
        error.to_string(),
        "wire input contains 9 bytes, exceeding the 8-byte limit"
    );
    assert!(error.source().is_none());
}

#[test]
fn test_value_wire_decode_error_preserves_json_source() {
    let source = serde_json::from_slice::<serde_json::Value>(b"not JSON")
        .expect_err("malformed JSON should fail");
    let error = ValueWireDecodeError::from(source);

    assert!(matches!(error, ValueWireDecodeError::InvalidJson(_)));
    assert!(error.source().is_some());
}
