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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_map(NamedValueWireOwnedVisitor)
        } else {
            deserializer.deserialize_struct("NamedValueWireOwned", &["name", "value"], NamedValueWireOwnedVisitor)
        }
    }
}
