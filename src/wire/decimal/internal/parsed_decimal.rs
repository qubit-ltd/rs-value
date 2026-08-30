// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned canonical decimal parsed for collection adapters.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;

use super::DecimalVisitor;

/// Deserializable wrapper used by canonical decimal collection adapters.
///
/// # Type Parameters
///
/// * `T` - Decimal-backed type stored after canonical deserialization.
pub(in crate::wire::decimal) struct ParsedDecimal<T>(
    /// Parsed decimal value.
    pub(in crate::wire::decimal) T,
);

impl<'de, T> Deserialize<'de> for ParsedDecimal<T>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
{
    /// Deserializes one canonical decimal string.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor(PhantomData)).map(Self)
    }
}
