// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Canonical Serde adapter for collections of string maps.

use std::collections::HashMap;

use serde::ser::SerializeSeq;
use serde::{
    Deserialize,
    Deserializer,
    Serializer,
};

use super::internal::CanonicalStringMap;

/// Serializes string maps in a collection with dictionary-ordered keys.
pub(crate) fn serialize<S>(
    values: &[HashMap<String, String>],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut sequence = serializer.serialize_seq(Some(values.len()))?;
    for value in values {
        sequence.serialize_element(&CanonicalStringMap(value))?;
    }
    sequence.end()
}

/// Deserializes a collection of string maps.
pub(crate) fn deserialize<'de, D>(
    deserializer: D,
) -> Result<Vec<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<HashMap<String, String>>::deserialize(deserializer)
}
