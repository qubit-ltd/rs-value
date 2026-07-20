// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Visitor for canonical wide integer strings.

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{self, Visitor};

use crate::wide_integer::parse_canonical_integer;

/// Parses one decimal string into an integer without retaining input text.
pub(in crate::wide_integer) struct IntegerVisitor<T>(
    /// Requested integer type.
    pub(in crate::wide_integer) PhantomData<T>,
);

impl<'de, T> Visitor<'de> for IntegerVisitor<T>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
{
    type Value = T;

    /// Describes the canonical input accepted by this visitor.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a base-10 128-bit integer string")
    }

    /// Parses one borrowed canonical integer string.
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse_canonical_integer(value)
    }

    /// Parses one owned canonical integer string.
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}
