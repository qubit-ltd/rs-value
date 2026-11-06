// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed display wrapper for canonical wide integers.

use std::fmt;

use serde::Serialize;
use serde::Serializer;

/// Serializes a borrowed integer using its display representation.
///
/// The wide-integer adapters supply integer types whose display output is
/// canonical decimal text. Allocation depends on the destination serializer's
/// `collect_str` implementation; the wrapper does not own the integer.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the borrowed integer.
/// * `T` - Displayable integer type serialized as canonical text.
pub(in crate::wide_integer) struct DisplayInteger<'a, T>(
    /// Borrowed integer to serialize.
    pub(in crate::wide_integer) &'a T,
);

impl<T> Serialize for DisplayInteger<'_, T>
where
    T: fmt::Display,
{
    /// Serializes the wrapped integer through its display representation.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Destination serializer type.
    ///
    /// # Parameters
    ///
    /// * `serializer` - Destination serializer consumed to emit the integer
    ///   text.
    ///
    /// # Returns
    ///
    /// The destination serializer's success value after emitting the text.
    ///
    /// # Errors
    ///
    /// Returns `S::Error` if the destination serializer cannot serialize the
    /// text.
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self.0)
    }
}
