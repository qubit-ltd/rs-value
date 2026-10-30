// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Fixed-capacity formatting for integer number text.

use std::fmt;

/// Maximum number of bytes needed for the decimal text of any `i64` or `u64`.
///
/// Twenty bytes cover `i64::MIN` (including its sign) and `u64::MAX`.
const MAX_INTEGER_TEXT_LENGTH: usize = 20;

/// Stores formatted integer text without allocating a `String`.
pub(in crate::wire) struct FixedNumberText {
    /// Bytes written so far, followed by unused capacity.
    bytes: [u8; MAX_INTEGER_TEXT_LENGTH],
    /// Number of initialized bytes in `bytes`.
    length: usize,
}

impl FixedNumberText {
    /// Creates an empty fixed-capacity text buffer.
    pub(in crate::wire) fn new() -> Self {
        Self {
            bytes: [0; MAX_INTEGER_TEXT_LENGTH],
            length: 0,
        }
    }

    /// Borrows the initialized text bytes.
    pub(in crate::wire) fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}

impl fmt::Write for FixedNumberText {
    /// Appends text when it fits in the fixed buffer.
    ///
    /// Returns `fmt::Error` if the capacity calculation overflows or the text
    /// does not fit. In that case, no bytes are written.
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let Some(end) = self.length.checked_add(value.len()) else {
            return Err(fmt::Error);
        };
        let Some(destination) = self.bytes.get_mut(self.length..end) else {
            return Err(fmt::Error);
        };
        destination.copy_from_slice(value.as_bytes());
        self.length = end;
        Ok(())
    }
}
