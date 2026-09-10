// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Length measurements for scalar Wire V1 representations.

use super::json_length_writer::JsonLengthWriter;

/// Returns the exact UTF-8 length of an integer's decimal representation.
pub(crate) fn decimal_len(value: i128) -> usize {
    let magnitude = value.unsigned_abs();
    let digits = if magnitude < 10 {
        1
    } else {
        magnitude.ilog10() as usize + 1
    };
    digits + usize::from(value < 0)
}

/// Returns the exact UTF-8 length of an unsigned integer's decimal form.
pub(crate) fn unsigned_decimal_len(value: u128) -> usize {
    if value < 10 { 1 } else { value.ilog10() as usize + 1 }
}

/// Counts a finite f32 using serde_json's compact formatter.
#[cfg(feature = "json")]
pub(crate) fn json_float32_len(value: f32) -> usize {
    use serde_json::ser::CompactFormatter;
    use serde_json::ser::Formatter;

    debug_assert!(value.is_finite());
    let mut writer = JsonLengthWriter::default();
    CompactFormatter
        .write_f32(&mut writer, value)
        .expect("one finite f32 lexeme fits usize");
    writer.len
}

/// Counts a finite f64 using serde_json's compact formatter.
#[cfg(feature = "json")]
pub(crate) fn json_float64_len(value: f64) -> usize {
    use serde_json::ser::CompactFormatter;
    use serde_json::ser::Formatter;

    debug_assert!(value.is_finite());
    let mut writer = JsonLengthWriter::default();
    CompactFormatter
        .write_f64(&mut writer, value)
        .expect("one finite f64 lexeme fits usize");
    writer.len
}
