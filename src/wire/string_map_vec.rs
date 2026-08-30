// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow source-test-pair
//! Canonical Serde adapter for collections of string maps.

use std::collections::HashMap;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serializer;
use serde::ser::SerializeSeq;

use super::internal::CanonicalStringMap;
use super::internal::StrictStringMap;

/// Serializes string maps in a collection with dictionary-ordered keys.
pub(crate) fn serialize<S>(values: &[HashMap<String, String>], serializer: S) -> Result<S::Ok, S::Error>
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
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<StrictStringMap<String>>::deserialize(deserializer)
        .map(|values| values.into_iter().map(StrictStringMap::into_inner).collect())
}
