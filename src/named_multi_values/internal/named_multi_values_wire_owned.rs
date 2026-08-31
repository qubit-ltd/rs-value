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
use serde::de::DeserializeSeed;

use crate::{ValueWireV1, ValueWireV1Seed};

fn deserialize_value_wire<'de, D>(deserializer: D) -> Result<ValueWireV1, D::Error>
where
    D: Deserializer<'de>,
{
    ValueWireV1Seed::new().deserialize(deserializer)
}

/// Owned wire representation of a named collection.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::named_multi_values) struct NamedMultiValuesWireOwned {
    /// Name associated with the collection.
    pub(in crate::named_multi_values) name: String,
    /// Independently versioned collection.
    #[serde(deserialize_with = "deserialize_value_wire")]
    pub(in crate::named_multi_values) value: ValueWireV1,
}
