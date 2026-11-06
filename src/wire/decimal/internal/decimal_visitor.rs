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

use serde::de;
use serde::de::Visitor;

use crate::wire::decimal::parse_canonical_decimal;

/// Parses one canonical decimal string into the requested value type.
///
/// # Type Parameters
///
/// * `T` - Decimal-backed type parsed from canonical text.
pub(in crate::wire::decimal) struct DecimalVisitor<T>(
    /// Requested decimal type.
    pub(in crate::wire::decimal) PhantomData<T>,
);

impl<'de, T> Visitor<'de> for DecimalVisitor<T>
where
    T: FromStr + fmt::Display,
    T::Err: fmt::Display,
{
    /// Decimal-backed value produced after canonical-text validation.
    type Value = T;

    /// Describes the textual input accepted by this visitor.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination for the expected-input description.
    ///
    /// # Returns
    ///
    /// Success after writing the description.
    ///
    /// # Errors
    ///
    /// Returns the destination's formatting error if writing fails.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a decimal string")
    }

    /// Parses one borrowed canonical decimal string.
    ///
    /// The parsed value must render back to exactly the input text.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Deserializer error type used to report invalid input.
    ///
    /// # Parameters
    ///
    /// * `value` - Candidate canonical decimal representation.
    ///
    /// # Returns
    ///
    /// The parsed decimal-backed value.
    ///
    /// # Errors
    ///
    /// Returns `E` when parsing fails or the representation is not canonical.
    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse_canonical_decimal(value)
    }

    /// Parses one owned canonical decimal string.
    ///
    /// The parsed value must render back to exactly the input text. The owned
    /// string is dropped after parsing.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Deserializer error type used to report invalid input.
    ///
    /// # Parameters
    ///
    /// * `value` - Candidate canonical decimal representation.
    ///
    /// # Returns
    ///
    /// The parsed decimal-backed value.
    ///
    /// # Errors
    ///
    /// Returns `E` when parsing fails or the representation is not canonical.
    #[inline]
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}
