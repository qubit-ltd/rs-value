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
use serde::de::DeserializeSeed;

use crate::ValueWireV1;
use crate::ValueWireV1Seed;

fn deserialize_value_wire<'de, D>(deserializer: D) -> Result<ValueWireV1, D::Error>
where
    D: Deserializer<'de>,
{
    ValueWireV1Seed::new().deserialize(deserializer)
}

/// Owned wire representation of a named scalar value.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::named_value) struct NamedValueWireOwned {
    /// Name associated with the scalar value.
    pub(in crate::named_value) name: String,
    /// Independently versioned scalar value.
    #[serde(deserialize_with = "deserialize_value_wire")]
    pub(in crate::named_value) value: ValueWireV1,
}
