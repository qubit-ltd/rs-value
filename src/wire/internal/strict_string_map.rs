// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strict string-map payload for canonical wire adapters.

use std::collections::HashMap;

use serde::Deserialize;
use serde::Deserializer;

use super::strict_string_map_visitor::StrictStringMapVisitor;

/// Owned string map decoded with duplicate-key validation.
///
/// # Type Parameters
///
/// * `V` - Deserialized map value type.
pub(in crate::wire) struct StrictStringMap<V>(
    /// Entries accumulated while rejecting duplicate keys.
    HashMap<String, V>,
);

impl<V> StrictStringMap<V> {
    /// Returns the validated map.
    ///
    /// # Parameters
    ///
    /// * `self` - The owned wrapper containing entries accepted by Serde.
    ///
    /// # Returns
    ///
    /// The owned map after duplicate-key validation has succeeded.
    pub(in crate::wire) fn into_inner(self) -> HashMap<String, V> {
        self.0
    }
}

impl<'de, V> Deserialize<'de> for StrictStringMap<V>
where
    V: Deserialize<'de>,
{
    /// Deserializes a string map while rejecting duplicate keys.
    ///
    /// # Type Parameters
    ///
    /// * `D` - Deserializer providing the map representation.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source of the string-map representation.
    ///
    /// # Returns
    ///
    /// A wrapper containing the decoded map when all keys are unique.
    ///
    /// # Errors
    ///
    /// Returns the deserializer error if a key or value cannot be decoded, or
    /// if the input contains a duplicate key.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(StrictStringMapVisitor::new()).map(Self)
    }
}
