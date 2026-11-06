// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned wire representation for one named collection.

use serde::Deserialize;
use serde::Deserializer;

use super::named_multi_values_wire_owned_visitor::NamedMultiValuesWireOwnedVisitor;
use crate::ValueWireV1;

/// Owned wire representation of a named collection.
pub(in crate::named_multi_values) struct NamedMultiValuesWireOwned {
    /// Name associated with the collection.
    pub(in crate::named_multi_values) name: String,
    /// Independently versioned collection.
    pub(in crate::named_multi_values) value: ValueWireV1,
}

impl<'de> Deserialize<'de> for NamedMultiValuesWireOwned {
    /// Decodes an owned name and versioned value using named fields for
    /// human-readable formats; binary formats may also use ordered fields.
    ///
    /// # Type Parameters
    ///
    /// * `D` - Deserializer supplying the record and its format policy.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source of the named wire record, consumed by
    ///   decoding.
    ///
    /// # Returns
    ///
    /// The owned name and decoded V1 value, before collection-shape validation
    /// by `NamedMultiValues`.
    ///
    /// # Errors
    ///
    /// Returns `D::Error` for missing, duplicate, unknown or malformed fields,
    /// invalid V1 values, or disallowed sequence representations.
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let allow_sequence = !deserializer.is_human_readable();
        deserializer.deserialize_struct(
            "NamedMultiValuesWireRef",
            &["name", "value"],
            NamedMultiValuesWireOwnedVisitor::new(allow_sequence),
        )
    }
}
