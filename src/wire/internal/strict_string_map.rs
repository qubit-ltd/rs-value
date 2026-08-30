// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow source-test-pair
//! Strict string-map payload for canonical wire adapters.

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::Deserializer;
use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;

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
    /// # Returns
    ///
    /// The owned map after duplicate-key validation succeeds.
    pub(in crate::wire) fn into_inner(self) -> HashMap<String, V> {
        self.0
    }
}

impl<'de, V> Deserialize<'de> for StrictStringMap<V>
where
    V: Deserialize<'de>,
{
    /// Deserializes a string map while rejecting duplicate keys.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrictStringMapVisitor<V>(PhantomData<V>);

        impl<'de, V> Visitor<'de> for StrictStringMapVisitor<V>
        where
            V: Deserialize<'de>,
        {
            type Value = StrictStringMap<V>;

            /// Describes the expected input shape.
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with unique string keys")
            }

            /// Decodes all entries and rejects repeated keys.
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
                Ok(StrictStringMap(values))
            }
        }

        deserializer.deserialize_map(StrictStringMapVisitor(PhantomData))
    }
}
