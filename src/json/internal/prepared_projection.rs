// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Admitted whole-value projection preserving scalar and collection shape.

use serde_json::Value as JsonValue;

use super::prepared_scalar::PreparedScalar;
use crate::MultiValuesRef;

/// Whole-operation preparation, avoiding an intermediate vector for simple
/// arrays.
///
/// # Type Parameters
///
/// - `'a`: lifetime of source storage retained by borrowed payloads.
pub(in crate::json) enum PreparedProjection<'a> {
    /// One scalar prepared without changing its shape.
    Scalar(
        /// Admitted scalar whose cached rendering is moved into the output.
        PreparedScalar<'a>,
    ),
    /// Admitted homogeneous storage requiring no per-element cache.
    BorrowedCollection(
        /// Source view retained until materialization; unset storage yields
        /// null.
        MultiValuesRef<'a>,
    ),
    /// Rich elements whose rendering must be retained.
    Collection(
        /// Owned preparations consumed in source order to produce the JSON
        /// array.
        Vec<PreparedScalar<'a>>,
    ),
}

impl PreparedProjection<'_> {
    /// Materializes an already admitted complete value, preserving null and
    /// array shapes.
    ///
    /// # Returns
    ///
    /// The complete JSON value represented by this prepared projection.
    ///
    /// # Panics
    ///
    /// Panics if a borrowed scalar requires formatting, violating the
    /// preparation invariant, or an index below a collection's reported
    /// length cannot be read.
    #[must_use]
    pub(in crate::json) fn materialize(self) -> JsonValue {
        match self {
            Self::Scalar(value) => value.materialize(),
            Self::Collection(values) => JsonValue::Array(
                values
                    .into_iter()
                    .map(PreparedScalar::materialize)
                    .collect(),
            ),
            Self::BorrowedCollection(MultiValuesRef::Unset(_)) => JsonValue::Null,
            Self::BorrowedCollection(values) => JsonValue::Array(
                (0..values.len())
                    .map(|index| {
                        let value = values
                            .get(index)
                            .expect("index is inside the admitted collection");
                        PreparedScalar::Borrowed(value).materialize()
                    })
                    .collect(),
            ),
        }
    }
}
