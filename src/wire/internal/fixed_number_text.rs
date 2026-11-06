// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Fixed-capacity formatting for JSON number text.

use std::fmt;
use std::str::from_utf8;

/// Capacity for common JSON number text, including finite `f64` values.
/// Longer arbitrary precision numbers fall back to an allocated string.
const MAX_NUMBER_TEXT_LENGTH: usize = 64;

/// Stores UTF-8 number text in a fixed-capacity stack buffer.
///
/// Appending a fragment either succeeds completely or leaves the text
/// unchanged.
pub(in crate::wire) struct FixedNumberText {
    /// Bytes written so far, followed by unused capacity.
    bytes: [u8; MAX_NUMBER_TEXT_LENGTH],
    /// Number of initialized bytes in `bytes`.
    length: usize,
}

impl FixedNumberText {
    /// Creates an empty fixed-capacity text buffer.
    ///
    /// # Returns
    ///
    /// A buffer with no initialized text and all capacity available.
    #[must_use]
    #[inline]
    pub(in crate::wire) fn new() -> Self {
        Self {
            bytes: [0; MAX_NUMBER_TEXT_LENGTH],
            length: 0,
        }
    }

    /// Borrows the initialized text bytes without allocating.
    ///
    /// # Returns
    ///
    /// The written UTF-8 bytes, valid for the lifetime of the buffer borrow.
    #[must_use]
    #[inline]
    pub(in crate::wire) fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }

    /// Borrows the initialized text as UTF-8 without allocating.
    ///
    /// # Returns
    ///
    /// The written text, valid for the lifetime of the buffer borrow.
    ///
    /// # Panics
    ///
    /// Panics if the internal UTF-8 invariant is violated. All writes preserve
    /// it.
    #[must_use]
    #[inline]
    pub(in crate::wire) fn as_str(&self) -> &str {
        from_utf8(self.as_bytes()).expect("formatted text must be valid UTF-8")
    }
}

impl fmt::Write for FixedNumberText {
    /// Appends text when it fits in the fixed buffer.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 fragment to append in full.
    ///
    /// # Returns
    ///
    /// `Ok(())` after appending the entire fragment.
    ///
    /// # Errors
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

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::FixedNumberText;

    #[test]
    fn test_accepts_text_that_exactly_fills_capacity() {
        let mut text = FixedNumberText::new();
        text.write_str(&"x".repeat(64)).unwrap();
        assert_eq!(text.as_str().len(), 64, "exact-capacity text must fit");
    }

    #[test]
    fn test_rejects_overflow_without_changing_written_text() {
        let mut text = FixedNumberText::new();
        text.write_str("prefix").unwrap();
        assert!(
            text.write_str(&"x".repeat(59)).is_err(),
            "a fragment exceeding remaining capacity must fail"
        );
        assert_eq!(text.as_str(), "prefix", "overflow must preserve written text");
    }

    #[test]
    fn test_appends_multiple_fragments() {
        let mut text = FixedNumberText::new();
        text.write_str("-1.25e").unwrap();
        text.write_str("10").unwrap();
        assert_eq!(text.as_str(), "-1.25e10", "fragments must append in order");
    }
}
