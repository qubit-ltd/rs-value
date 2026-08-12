// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow source-test-pair
//! Canonical Serde adapter for string maps.

use std::collections::HashMap;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serializer;
use serde::ser::SerializeMap;

use super::internal::StrictStringMap;

/// Serializes one string map with lexicographically ordered keys.
pub(crate) fn serialize<S>(
    value: &HashMap<String, String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut entries: Vec<_> = value.iter().collect();
    entries.sort_unstable_by_key(|(left, _)| *left);

    let mut map = serializer.serialize_map(Some(entries.len()))?;
    for (key, value) in entries {
        map.serialize_entry(key, value)?;
    }
    map.end()
}

/// Deserializes one string map from any Serde map representation.
pub(crate) fn deserialize<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    StrictStringMap::<String>::deserialize(deserializer)
        .map(StrictStringMap::into_inner)
}
