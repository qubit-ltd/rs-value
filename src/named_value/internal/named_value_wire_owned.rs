// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned wire representation for one named scalar value.

use serde::Deserialize;
use serde::Deserializer;

use super::named_value_wire_owned_visitor::NamedValueWireOwnedVisitor;
use crate::ValueWireV1;

/// Owned wire representation of a named scalar value.
pub(in crate::named_value) struct NamedValueWireOwned {
    /// Name associated with the scalar value.
    pub(in crate::named_value) name: String,
    /// Independently versioned scalar value.
    pub(in crate::named_value) value: ValueWireV1,
}

impl<'de> Deserialize<'de> for NamedValueWireOwned {
    /// Deserializes an owned named scalar record using the format's struct
    /// encoding.
    ///
    /// Human-readable formats must use named fields; binary formats may also
    /// use an ordered sequence containing the name followed by the
    /// versioned value.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Deserializer supplying the record and its format policy.
    ///
    /// # Parameters
    ///
    /// - `deserializer`: Consumed input from which both owned fields are
    ///   decoded.
    ///
    /// # Returns
    ///
    /// The decoded record with an owned name and versioned scalar value.
    ///
    /// # Errors
    ///
    /// Returns a deserializer error for malformed records, missing, duplicate,
    /// or unknown fields, unsupported scalar values, or disallowed sequences.
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let allow_sequence = !deserializer.is_human_readable();
        deserializer.deserialize_struct(
            "NamedValueWireRef",
            &["name", "value"],
            NamedValueWireOwnedVisitor::new(allow_sequence),
        )
    }
}
