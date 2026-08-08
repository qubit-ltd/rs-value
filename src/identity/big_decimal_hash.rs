// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Canonical hashing for [`bigdecimal::BigDecimal`].

use std::hash::Hash;
use std::hash::Hasher;

use bigdecimal::BigDecimal;
use num_bigint::Sign;

/// Hashes a decimal by its normalized coefficient and effective scale.
///
/// The work and temporary allocation are proportional to the stored
/// coefficient digit count and do not depend on the absolute scale.
///
/// # Parameters
///
/// * `value` - Decimal whose representation is normalized for hashing.
/// * `state` - Destination hasher.
pub(crate) fn hash_big_decimal<H: Hasher>(value: &BigDecimal, state: &mut H) {
    let (coefficient, scale) = value.as_bigint_and_exponent();
    if coefficient.sign() == Sign::NoSign {
        0_u8.hash(state);
        return;
    }

    let coefficient = coefficient.to_str_radix(10);
    let normalized = coefficient.trim_end_matches('0');
    let trailing_zero_count = coefficient.len() - normalized.len();
    let effective_scale = i128::from(scale) - trailing_zero_count as i128;

    1_u8.hash(state);
    normalized.hash(state);
    effective_scale.hash(state);
}
