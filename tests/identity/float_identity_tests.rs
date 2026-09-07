// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests floating-point identity normalization.

use qubit_value::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// Verifies NaN payloads retain reflexive public value identity.
#[test]
fn test_float_identity_normalizes_nan_payloads() {
    let left = Value::Float64(f64::from_bits(0x7ff8_0000_0000_0001));
    let right = Value::Float64(f64::from_bits(0x7fff_ffff_ffff_ffff));
    assert_eq!(left, right);
}

/// Verifies signed zero has one identity and one hash representation.
#[test]
fn test_float_identity_normalizes_signed_zero_hashes() {
    let left = Value::Float64(0.0);
    let right = Value::Float64(-0.0);
    let mut left_hasher = DefaultHasher::new();
    let mut right_hasher = DefaultHasher::new();

    left.hash(&mut left_hasher);
    right.hash(&mut right_hasher);

    assert_eq!(left, right);
    assert_eq!(left_hasher.finish(), right_hasher.finish());
}
