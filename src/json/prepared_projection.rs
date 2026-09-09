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
pub(super) enum PreparedProjection<'a> {
    /// One scalar prepared without changing its shape.
    Scalar(PreparedScalar<'a>),
    /// Admitted homogeneous storage requiring no per-element cache.
    BorrowedCollection(MultiValuesRef<'a>),
    /// Rich elements whose rendering must be retained.
    Collection(Vec<PreparedScalar<'a>>),
}

impl PreparedProjection<'_> {
    /// Materializes an already admitted complete value, preserving null and
    /// array shapes.
    pub(super) fn materialize(self) -> JsonValue {
        match self {
            Self::Scalar(value) => value.materialize(),
            Self::Collection(values) => JsonValue::Array(values.into_iter().map(PreparedScalar::materialize).collect()),
            Self::BorrowedCollection(MultiValuesRef::Unset(_)) => JsonValue::Null,
            Self::BorrowedCollection(values) => JsonValue::Array(
                (0..values.len())
                    .map(|index| {
                        let value = values.get(index).expect("index is inside the admitted collection");
                        PreparedScalar::Borrowed(value).materialize()
                    })
                    .collect(),
            ),
        }
    }
}
