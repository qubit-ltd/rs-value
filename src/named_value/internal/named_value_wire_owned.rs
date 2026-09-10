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

/// Decodes the nested V1 envelope with its explicit validation seed.
///
/// # Type Parameters
///
/// * `D` - Serde deserializer supplied by the enclosing named-value visitor.
///
/// # Parameters
///
/// * `deserializer` - Source positioned at the nested V1 envelope.
///
/// # Returns
///
/// A validated V1 wire value whose resource accounting is inherited from the
/// enclosing deserializer.
///
/// # Errors
///
/// Returns `D::Error` when the nested V1 envelope is malformed or unsupported.
#[inline(always)]
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
