// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned canonical wide integer parsed for collection adapters.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;

use super::IntegerVisitor;

/// Deserializable wrapper used by collection adapters.
///
/// # Type Parameters
///
/// * `T` - Integer type stored after canonical deserialization.
pub(in crate::wide_integer) struct ParsedInteger<T>(
    /// Parsed integer value.
    pub(in crate::wide_integer) T,
);

impl<'de, T> Deserialize<'de> for ParsedInteger<T>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
{
    /// Deserializes one canonical integer string.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IntegerVisitor(PhantomData)).map(Self)
    }
}
