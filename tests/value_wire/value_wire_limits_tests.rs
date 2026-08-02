// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for `ValueWireLimits`.

use qubit_value::{ValueWireDecodeError, ValueWireLimits};

#[test]
fn test_value_wire_limits_default_uses_documented_byte_budget() {
    let limits = ValueWireLimits::default();

    assert_eq!(
        limits.max_json_bytes(),
        ValueWireLimits::DEFAULT_MAX_JSON_BYTES
    );
    assert_eq!(limits.max_json_bytes(), 1_048_576);
}

#[test]
fn test_value_wire_limits_new_preserves_custom_byte_budget() {
    let limits = ValueWireLimits::new(64 * 1024);

    assert_eq!(limits.max_json_bytes(), 65_536);
}

#[test]
fn test_value_wire_limits_check_json_bytes_enforces_public_budget() {
    let limits = ValueWireLimits::new(8);

    limits
        .check_json_bytes(8)
        .expect("input at the byte budget should be accepted");
    assert!(matches!(
        limits.check_json_bytes(9),
        Err(ValueWireDecodeError::InputTooLarge {
            input_bytes: 9,
            max_input_bytes: 8,
        })
    ));
}
