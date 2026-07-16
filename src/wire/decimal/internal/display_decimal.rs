// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed display wrapper for canonical decimal values.

use std::fmt;

use serde::{
    Serialize,
    Serializer,
};

/// Serializes a decimal value through its stable textual form.
pub(in crate::wire::decimal) struct DisplayDecimal<'a, T>(
    /// Borrowed decimal value to serialize.
    pub(in crate::wire::decimal) &'a T,
);

impl<T> Serialize for DisplayDecimal<'_, T>
where
    T: fmt::Display,
{
    /// Serializes the wrapped decimal through its display representation.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self.0)
    }
}
