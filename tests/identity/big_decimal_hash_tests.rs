// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests canonical decimal hashing through the public value API.

use std::collections::hash_map::DefaultHasher;
use std::hash::{
    Hash,
    Hasher,
};

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_value::Value;

/// Verifies equivalent decimal encodings have identical hashes.
#[test]
fn test_big_decimal_hash_normalizes_equivalent_encodings() {
    let left = Value::BigDecimal(BigDecimal::new(BigInt::from(1), 0));
    let right = Value::BigDecimal(BigDecimal::new(BigInt::from(10), 1));
    let mut left_hasher = DefaultHasher::new();
    let mut right_hasher = DefaultHasher::new();
    left.hash(&mut left_hasher);
    right.hash(&mut right_hasher);
    assert_eq!(left, right);
    assert_eq!(left_hasher.finish(), right_hasher.finish());
}
