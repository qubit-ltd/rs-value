// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Length measurements for scalar Wire V1 representations.

#[cfg(feature = "json")]
use serde_json::ser::CompactFormatter;
#[cfg(feature = "json")]
use serde_json::ser::Formatter;

use super::json_length_writer::JsonLengthWriter;

/// Returns the UTF-8 byte length of an integer's base-10 representation,
/// including its minus sign when the value is negative.
#[must_use]
#[inline]
pub(crate) fn decimal_len(value: i128) -> usize {
    let magnitude = value.unsigned_abs();
    let digits = if magnitude < 10 {
        1
    } else {
        magnitude.ilog10() as usize + 1
    };
    digits + usize::from(value < 0)
}

/// Returns the UTF-8 byte length of an unsigned integer's base-10
/// representation.
#[must_use]
#[inline]
pub(crate) fn unsigned_decimal_len(value: u128) -> usize {
    if value < 10 { 1 } else { value.ilog10() as usize + 1 }
}

/// Returns the byte length serde_json's compact formatter writes for a finite
/// `f32`.
///
/// The caller must handle non-finite values separately because JSON has no
/// numeric representation for them.
#[must_use]
#[cfg(feature = "json")]
pub(crate) fn json_float32_len(value: f32) -> usize {
    debug_assert!(value.is_finite());
    let mut writer = JsonLengthWriter::default();
    CompactFormatter
        .write_f32(&mut writer, value)
        .expect("one finite f32 lexeme fits usize");
    writer.len
}

/// Returns the byte length serde_json's compact formatter writes for a finite
/// `f64`.
///
/// The caller must handle non-finite values separately because JSON has no
/// numeric representation for them.
#[must_use]
#[cfg(feature = "json")]
pub(crate) fn json_float64_len(value: f64) -> usize {
    debug_assert!(value.is_finite());
    let mut writer = JsonLengthWriter::default();
    CompactFormatter
        .write_f64(&mut writer, value)
        .expect("one finite f64 lexeme fits usize");
    writer.len
}
