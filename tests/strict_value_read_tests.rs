// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests strict scalar reads.

use qubit_value::Value;

/// Verifies strict reads return the value when the requested type matches.
#[test]
fn test_strict_value_read_returns_matching_value() {
    let value = Value::Int32(7);
    assert_eq!(value.get::<i32>().expect("matching type must be readable"), 7);
}
