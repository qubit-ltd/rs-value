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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_map(NamedMultiValuesWireOwnedVisitor)
        } else {
            deserializer.deserialize_struct(
                "NamedMultiValuesWireOwned",
                &["name", "value"],
                NamedMultiValuesWireOwnedVisitor,
            )
        }
    }
}
