// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed runtime shape used by V1 serialization.

use serde::Serialize;

use super::CollectionWireRef;
use super::ScalarWireRef;
use crate::ValueContainer;

/// Borrowed scalar-or-collection shape used during serialization.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the borrowed scalar or collection payload.
#[derive(Clone, Copy, Serialize)]
pub(in crate::value_wire) enum WireShapeRef<'a> {
    /// One typed scalar.
    #[serde(rename = "scalar")]
    Scalar(
        /// Borrowed scalar payload.
        ScalarWireRef<'a>,
    ),
    /// One homogeneous typed collection.
    #[serde(rename = "collection")]
    Collection(
        /// Borrowed collection payload.
        CollectionWireRef<'a>,
    ),
}

impl<'a> From<&'a ValueContainer> for WireShapeRef<'a> {
    /// Borrows the explicit runtime shape for V1 serialization.
    #[inline]
    fn from(value: &'a ValueContainer) -> Self {
        match value {
            ValueContainer::Scalar(value) => Self::Scalar(value.into()),
            ValueContainer::Collection(values) => Self::Collection(values.into()),
        }
    }
}
