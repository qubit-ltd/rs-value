// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned runtime shape used by V1 deserialization.

use serde::Deserialize;

use super::CollectionWireOwned;
use super::ScalarWireOwned;
use crate::ValueContainer;

/// Owned scalar-or-collection shape used during deserialization.
#[derive(Deserialize)]
pub(in crate::value_wire) enum WireShapeOwned {
    /// One typed scalar.
    #[serde(rename = "scalar")]
    Scalar(
        /// Owned scalar payload.
        ScalarWireOwned,
    ),
    /// One homogeneous typed collection.
    #[serde(rename = "collection")]
    Collection(
        /// Owned collection payload.
        CollectionWireOwned,
    ),
}

impl From<WireShapeOwned> for ValueContainer {
    /// Restores the explicit runtime scalar-or-collection shape.
    #[inline]
    fn from(value: WireShapeOwned) -> Self {
        match value {
            WireShapeOwned::Scalar(value) => Self::Scalar(value.into()),
            WireShapeOwned::Collection(values) => Self::Collection(values.into()),
        }
    }
}
