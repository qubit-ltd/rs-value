// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for `ValueWireLimits`.

use qubit_value::ValueWireLimits;

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
