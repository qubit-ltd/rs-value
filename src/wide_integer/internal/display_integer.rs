// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed display wrapper for canonical wide integers.

use std::fmt;

use serde::{
    Serialize,
    Serializer,
};

/// Serializes a displayable integer as a decimal string without allocating.
pub(in crate::wide_integer) struct DisplayInteger<'a, T>(
    /// Borrowed integer to serialize.
    pub(in crate::wide_integer) &'a T,
);

impl<T> Serialize for DisplayInteger<'_, T>
where
    T: fmt::Display,
{
    /// Serializes the wrapped integer through its display representation.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self.0)
    }
}
