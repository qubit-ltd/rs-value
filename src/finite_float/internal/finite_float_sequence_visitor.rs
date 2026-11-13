// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validates each decoded float before requesting the next typed element.

use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::de::Error as _;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::FiniteFloat;
use crate::finite_float::NON_FINITE_FLOAT_MESSAGE;

/// Builds an owned sequence containing only finite floats.
///
/// Failure stops typed element requests; outer document preflight and already
/// charged resources remain the caller's responsibility.
///
/// # Type Parameters
///
/// * `T` - Decoded element type; visiting requires deserialization support and
///   the shared finite-float predicate.
pub(in crate::finite_float) struct FiniteFloatSequenceVisitor<T> {
    /// Associates the visitor with its decoded float width without storing it.
    marker: PhantomData<fn() -> T>,
}

impl<T> FiniteFloatSequenceVisitor<T> {
    /// Creates a stateless visitor for the decoded float width.
    ///
    /// # Returns
    ///
    /// A visitor that stores no decoded elements until a sequence is visited.
    #[must_use]
    #[inline]
    pub(in crate::finite_float) const fn new() -> Self {
        Self { marker: PhantomData }
    }
}

impl<'de, T> Visitor<'de> for FiniteFloatSequenceVisitor<T>
where
    T: Deserialize<'de> + FiniteFloat,
{
    /// Owns the validated finite elements in their original input order.
    type Value = Vec<T>;

    /// Describes the sequence representation required by the adapter.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination for the expected-representation description.
    ///
    /// # Returns
    ///
    /// Success after writing the description.
    ///
    /// # Errors
    ///
    /// Returns the destination's formatting error if writing fails.
    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str("a sequence of finite floating-point values")
    }

    /// Returns decoded finite values in input order without reading past an
    /// error.
    ///
    /// # Type Parameters
    ///
    /// * `A` - Serde sequence access supplying elements for the input lifetime.
    ///
    /// # Parameters
    ///
    /// * `sequence` - Consumed access to the sequence being decoded.
    ///
    /// # Returns
    ///
    /// An owned vector of finite values in input order, empty for an empty
    /// sequence.
    ///
    /// # Errors
    ///
    /// Returns the source error when an element cannot be decoded, or the
    /// stable non-finite error before requesting the following typed element.
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<T>()? {
            if !value.is_finite() {
                return Err(A::Error::custom(NON_FINITE_FLOAT_MESSAGE));
            }
            values.push(value);
        }
        Ok(values)
    }
}
