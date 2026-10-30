// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compares formatted number text with borrowed expected bytes.

use std::fmt;

/// Compares each formatted text fragment against an expected byte slice.
pub(in crate::wire) struct NumberTextComparator<'a> {
    /// Expected bytes borrowed from the caller for this comparison.
    expected: &'a [u8],
    /// Number of expected bytes already matched.
    offset: usize,
}

impl<'a> NumberTextComparator<'a> {
    /// Creates a comparator borrowing the complete expected text.
    pub(in crate::wire) fn new(expected: &'a [u8]) -> Self {
        Self { expected, offset: 0 }
    }

    /// Returns whether all expected bytes have been matched.
    pub(in crate::wire) fn is_equal(&self) -> bool {
        self.offset == self.expected.len()
    }
}

impl fmt::Write for NumberTextComparator<'_> {
    /// Compares one formatted fragment at the current byte offset.
    ///
    /// Returns `fmt::Error` when the fragment differs or exceeds the expected
    /// slice. A successful write advances the comparison offset.
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let Some(end) = self.offset.checked_add(value.len()) else {
            return Err(fmt::Error);
        };
        let Some(expected) = self.expected.get(self.offset..end) else {
            return Err(fmt::Error);
        };
        if expected != value.as_bytes() {
            return Err(fmt::Error);
        }
        self.offset = end;
        Ok(())
    }
}
