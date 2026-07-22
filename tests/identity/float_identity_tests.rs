// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests floating-point identity normalization.

use qubit_value::Value;

/// Verifies NaN payloads retain reflexive public value identity.
#[test]
fn test_float_identity_normalizes_nan_payloads() {
    let left = Value::Float64(f64::from_bits(0x7ff8_0000_0000_0001));
    let right = Value::Float64(f64::from_bits(0x7fff_ffff_ffff_ffff));
    assert_eq!(left, right);
}
