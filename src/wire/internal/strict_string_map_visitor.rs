// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Map visitor that rejects duplicate string keys during deserialization.

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;

/// Deserialization visitor that accumulates unique string-keyed entries.
///
/// # Type Parameters
///
/// * `V` - The value type deserialized for each string key.
pub(super) struct StrictStringMapVisitor<V>(PhantomData<V>);

impl<V> StrictStringMapVisitor<V> {
    /// Creates a visitor for values of type `V`.
    #[must_use]
    #[inline]
    pub(super) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, V> Visitor<'de> for StrictStringMapVisitor<V>
where
    V: Deserialize<'de>,
{
    type Value = HashMap<String, V>;

    /// Describes the expected input shape.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map with unique string keys")
    }

    /// Decodes all entries and rejects repeated keys.
    ///
    /// # Type Parameters
    ///
    /// * `A` - The deserializer's map access implementation.
    ///
    /// # Errors
    ///
    /// Returns the map access error if an entry cannot be decoded, or a custom
    /// deserialization error if a key appears more than once.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = HashMap::new();
        while let Some((key, value)) = map.next_entry::<String, V>()? {
            if values.insert(key.clone(), value).is_some() {
                return Err(de::Error::custom(format!("duplicate map key '{key}'")));
            }
        }
        Ok(values)
    }
}
