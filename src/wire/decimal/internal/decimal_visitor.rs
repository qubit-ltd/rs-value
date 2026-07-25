// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Visitor for canonical decimal strings.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{self, Visitor};

use crate::wire::decimal::parse_canonical_decimal;

/// Parses one canonical decimal string into the requested value type.
pub(in crate::wire::decimal) struct DecimalVisitor<T>(
    /// Requested decimal type.
    pub(in crate::wire::decimal) PhantomData<T>,
);

impl<'de, T> Visitor<'de> for DecimalVisitor<T>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
{
    type Value = T;

    /// Describes the textual input accepted by this visitor.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a decimal string")
    }

    /// Parses one borrowed canonical decimal string.
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse_canonical_decimal(value)
    }

    /// Parses one owned canonical decimal string.
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}
