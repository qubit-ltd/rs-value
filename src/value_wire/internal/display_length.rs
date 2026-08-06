// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow source-test-pair
//! Allocation-free measurement of canonical display payloads.

use std::fmt::{
    self,
    Display,
    Write,
};

/// Formatting sink that counts UTF-8 bytes without retaining output.
struct DisplayLength {
    /// Number of bytes written so far.
    bytes: usize,
}

impl Write for DisplayLength {
    /// Adds the supplied UTF-8 byte length to the running total.
    #[inline(always)]
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.bytes = self.bytes.saturating_add(value.len());
        Ok(())
    }
}

/// Returns the display length of `value` without allocating a string.
#[inline]
pub(in crate::value_wire) fn display_length(value: impl Display) -> usize {
    let mut output = DisplayLength { bytes: 0 };
    if write!(&mut output, "{value}").is_err() {
        return usize::MAX;
    }
    output.bytes
}
