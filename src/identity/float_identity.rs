// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Canonical identity bits for floating-point payloads.

/// Returns canonical identity bits for an `f32` payload.
///
/// # Parameters
///
/// * `value` - Floating-point payload to canonicalize.
///
/// # Returns
///
/// Positive-zero bits for either signed zero, one quiet-NaN representation for
/// every NaN, and the original bits for every other value.
#[must_use]
#[inline]
pub(crate) fn canonical_f32_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else if value.is_nan() {
        f32::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

/// Returns canonical identity bits for an `f64` payload.
///
/// # Parameters
///
/// * `value` - Floating-point payload to canonicalize.
///
/// # Returns
///
/// Positive-zero bits for either signed zero, one quiet-NaN representation for
/// every NaN, and the original bits for every other value.
#[must_use]
#[inline]
pub(crate) fn canonical_f64_bits(value: f64) -> u64 {
    if value == 0.0 {
        0.0_f64.to_bits()
    } else if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }
}
